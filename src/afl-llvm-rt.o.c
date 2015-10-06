/*
   american fuzzy lop - LLVM instrumentation bootstrap
   ---------------------------------------------------

   Written by Laszlo Szekeres <lszekeres@google.com>,
              Michal Zalewski <lcamtuf@google.com>, and
              Keegan McAllister <mcallister.keegan@gmail.com>

   LLVM integration design comes from Laszlo Szekeres.

   Copyright 2015 Google Inc. All rights reserved.

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at:

     http://www.apache.org/licenses/LICENSE-2.0

   This code is the rewrite of afl-as.h's main_payload.

*/

#include "config.h"

#include <stdlib.h>
#include <sys/mman.h>
#include <sys/shm.h>
#include <sys/wait.h>
#include <unistd.h>
#include <assert.h>
#include <stdint.h>

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

/* Globals needed by the injected instrumentation. The __afl_area_initial region
   is used for instrumentation output before __afl_map_shm() has a chance to run.
   It will end up as .comm, so it shouldn't be too wasteful. */

u8  __afl_area_initial[MAP_SIZE];
u8* __afl_area_ptr = __afl_area_initial;
u16 __afl_prev_loc;

/* SHM setup. */

static void __afl_map_shm(void) {

  char *id_str = getenv(SHM_ENV_VAR);

  /* If we're running under AFL, attach to the appropriate region, replacing the
     early-stage __afl_area_initial region that is needed to allow some really
     hacky .init code to work correctly in projects such as OpenSSL. */

  if (id_str) {

    u32 shm_id = atoi(id_str);

    __afl_area_ptr = shmat(shm_id, NULL, 0);

    /* Whooooops. */

    if (__afl_area_ptr == (void *)-1) exit(1);

    /* Write something into the bitmap so that even with low AFL_INST_RATIO,
       our parent doesn't give up on us. */

    __afl_area_ptr[0] = 1;

  }

}


/* Fork server logic. */

static void __afl_start_forkserver(void) {

  static u8 tmp[4];

  /* Phone home and tell the parent that we're OK. If parent isn't there,
     assume we're not running in forkserver mode and just execute program. */

  if (write(FORKSRV_FD + 1, tmp, 4) != 4) return;

  while (1) {

    s32 child_pid;
    int status;

    /* Wait for parent by reading from the pipe. Abort if read fails. */

    if (read(FORKSRV_FD, tmp, 4) != 4) exit(1);

    /* Once woken up, create a clone of our process. */

    child_pid = fork();
    if (child_pid < 0) exit(1);

    /* In child process: close fds, resume execution. */

    if (!child_pid) {

      close(FORKSRV_FD);
      close(FORKSRV_FD + 1);
      return;

    }

    /* In parent process: write PID to pipe, then wait for child. */

    if (write(FORKSRV_FD + 1, &child_pid, 4) != 4) exit(1);
    if (waitpid(child_pid, &status, WUNTRACED) < 0) exit(1);

    /* Relay wait status to pipe, then loop back. */

    if (write(FORKSRV_FD + 1, &status, 4) != 4) exit(1);

  }

}

/* This one can be called from user code when AFL_DEFER_FORKSRV is set. */

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

  if (getenv("AFL_DEFER_FORKSRV")) return;
  __afl_manual_init();

}
