/*
   american fuzzy lop - LLVM instrumentation bootstrap
   ---------------------------------------------------

   Written by Laszlo Szekeres <lszekeres@google.com>,
              Michal Zalewski <lcamtuf@google.com>, and
              Keegan McAllister <mcallister.keegan@gmail.com>

   LLVM integration design comes from Laszlo Szekeres.

   Copyright 2015, 2016 Google Inc. All rights reserved.

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at:

     http://www.apache.org/licenses/LICENSE-2.0

   This code is the rewrite of afl-as.h's main_payload.

*/

#include "config.h"

#include <stdio.h>
#include <stdlib.h>
#include <signal.h>
#include <unistd.h>
#include <assert.h>
#include <stdint.h>

#include <sys/mman.h>
#include <sys/shm.h>
#include <sys/wait.h>
#include <sys/types.h>

typedef uint8_t u8;
typedef uint16_t u16;
typedef uint32_t u32;

typedef int8_t s8;
typedef int16_t s16;
typedef int32_t s32;

/* OSX uses MAP_ANON instead of MAP_ANONYMOUS */
#ifndef MAP_ANONYMOUS
#  define MAP_ANONYMOUS MAP_ANON
#endif

#ifndef MIN
#  define MIN(_a,_b) ((_a) > (_b) ? (_b) : (_a))
#  define MAX(_a,_b) ((_a) > (_b) ? (_a) : (_b))
#endif /* !MIN */

/* Globals needed by the injected instrumentation. The __afl_area_initial region
   is used for instrumentation output before __afl_map_shm() has a chance to run.
   It will end up as .comm, so it shouldn't be too wasteful. */

u8  __afl_area_initial[MAP_SIZE];
u8* __afl_area_ptr = __afl_area_initial;

__thread u32 __afl_prev_loc;


/* Running in persistent mode? */

static u8 is_persistent;


/* SHM setup. */

static void __afl_map_shm(void) {

  u8 *id_str = getenv(SHM_ENV_VAR);

  /* If we're running under AFL, attach to the appropriate region, replacing the
     early-stage __afl_area_initial region that is needed to allow some really
     hacky .init code to work correctly in projects such as OpenSSL. */

  if (id_str) {

    u32 shm_id = atoi(id_str);

    __afl_area_ptr = shmat(shm_id, NULL, 0);

    /* Whooooops. */

    if (__afl_area_ptr == (void *)-1) _exit(1);

    /* Write something into the bitmap so that even with low AFL_INST_RATIO,
       our parent doesn't give up on us. */

    __afl_area_ptr[0] = 1;

  }

}


/* Fork server logic. */

static void __afl_start_forkserver(void) {

  static u8 tmp[4];
  s32 child_pid;

  u8  child_stopped = 0;

  /* Phone home and tell the parent that we're OK. If parent isn't there,
     assume we're not running in forkserver mode and just execute program. */

  if (write(FORKSRV_FD + 1, tmp, 4) != 4) return;

  while (1) {

    u32 was_killed;
    int status;

    /* Wait for parent by reading from the pipe. Abort if read fails. */

    if (read(FORKSRV_FD, &was_killed, 4) != 4) _exit(1);

    /* If we stopped the child in persistent mode, but there was a race
       condition and afl-fuzz already issued SIGKILL, write off the old
       process. */

    if (child_stopped && was_killed) {
      child_stopped = 0;
      if (waitpid(child_pid, &status, 0) < 0) _exit(1);
    }

    if (!child_stopped) {

      /* Once woken up, create a clone of our process. */

      child_pid = fork();
      if (child_pid < 0) _exit(1);

      /* In child process: close fds, resume execution. */

      if (!child_pid) {

        close(FORKSRV_FD);
        close(FORKSRV_FD + 1);
        return;

      }

    } else {

      /* Special handling for persistent mode: if the child is alive but
         currently stopped, simply restart it with SIGCONT. */

      kill(child_pid, SIGCONT);
      child_stopped = 0;

    }

    /* In parent process: write PID to pipe, then wait for child. */

    if (write(FORKSRV_FD + 1, &child_pid, 4) != 4) _exit(1);

    if (waitpid(child_pid, &status, is_persistent ? WUNTRACED : 0) < 0)
      _exit(1);

    /* In persistent mode, the child stops itself with SIGSTOP to indicate
       a successful run. In this case, we want to wake it up without forking
       again. */

    if (WIFSTOPPED(status)) child_stopped = 1;

    /* Relay wait status to pipe, then loop back. */

    if (write(FORKSRV_FD + 1, &status, 4) != 4) _exit(1);

  }

}


/* A simplified persistent mode handler, used as explained in README.llvm. */

int __afl_persistent_loop(unsigned int max_cnt) {

  static u8  first_pass = 1;
  static u32 cycle_cnt;

  if (first_pass) {

    cycle_cnt  = max_cnt;
    first_pass = 0;
    return 1;

  }

  if (is_persistent && --cycle_cnt) {

    raise(SIGSTOP);
    return 1;

  } else return 0;

}


/* This one can be called from user code when deferred forkserver mode
    is enabled. */

void __afl_manual_init(void) {

  static u8 init_done;

  if (!init_done) {

    __afl_map_shm();
    __afl_start_forkserver();
    init_done = 1;

  }

}


/* Proper initialization routine. */

__attribute__((constructor(0))) void __afl_auto_init(void) {

  is_persistent = !!getenv(PERSIST_ENV_VAR);

  if (getenv(DEFER_ENV_VAR)) return;

  __afl_manual_init();

}


/*********************************************
 * Support for -fsanitize-coverage=trace-pc. *
 *********************************************/

static u32 inst_ratio_scaled = MIN(4096, MAP_SIZE);


/* The first function is called on every basic block. We use the return address
   instead of a randomly-generated token (because LLVM is not giving us one).
   Since ASLR may make addresses vary across runs, we use only the last 12
   bits, which should be stable within a given binary.

   Since MAP_SIZE is usually larger than 12 bits, we "pad" it by combining
   left-shifted __afl_prev_loc. This gives us a theoretical maximum of 24
   bits (but basic blocks might be aligned, which reduces this number
   somewhat). */

void __sanitizer_cov_trace_pc(void) {

  u32 cur = ((u32)__builtin_return_address(0)) & MIN(4095, MAP_SIZE - 1);

  if (cur > inst_ratio_scaled) return;

  __afl_area_ptr[cur ^ __afl_prev_loc]++;

#if MAP_SIZE_POW2 > 12
  __afl_prev_loc = cur << (MAP_SIZE_POW2 - 12);
#else
  __afl_prev_loc = cur >> 1;
#endif /* ^MAP_SIZE_POW2 > 12 */

}


/* Same deal, but for indirect calls. */

void __sanitizer_cov_trace_pc_indir(void* dummy) {

  u32 cur = ((u32)__builtin_return_address(0)) & MIN(4095, MAP_SIZE - 1);

  if (cur > inst_ratio_scaled) return;

  __afl_area_ptr[cur ^ __afl_prev_loc]++;

#if MAP_SIZE_POW2 > 12
  __afl_prev_loc = cur << (MAP_SIZE_POW2 - 12);
#else
  __afl_prev_loc = cur >> 1;
#endif /* ^MAP_SIZE_POW2 > 12 */

}


/* Init callback. Unfortunately, LLVM does not support compile-time
   instrumentation density scaling, at least not just yet - so the runtime
   inst_ratio stuff slows us down :-( */

void __sanitizer_cov_module_init(void) {

  u8* x = getenv("AFL_INST_RATIO");

  if (!x) return;

  inst_ratio_scaled = atoi(x);

  if (!inst_ratio_scaled || inst_ratio_scaled > 100) {
    fprintf(stderr, "[-] ERROR: Invalid AFL_INST_RATIO (must be 1-100).\n");
    abort();
  }

  inst_ratio_scaled = inst_ratio_scaled * MIN(4096, MAP_SIZE) / 100;

}
