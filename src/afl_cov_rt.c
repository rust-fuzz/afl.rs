/*
   american fuzzy lop - LLVM instrumentation runtime
   -------------------------------------------------

   Written by Laszlo Szekeres <lszekeres@google.com>

   Copyright 2015 Google Inc. All rights reserved.

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at:

     http://www.apache.org/licenses/LICENSE-2.0

   This code is the rewrite of afl-as.h's main_payload. Some parts are taken
   from qemu_mode/patches/afl-qemu-cpu-inl.h.
*/

#include "config.h"
#include <sys/mman.h>
#include <sys/shm.h>
#include <sys/wait.h>
#include <unistd.h>
#include <stdlib.h>

/* OSX uses MAP_ANON instead of MAP_ANONYMOUS */
#ifndef MAP_ANONYMOUS
#  define MAP_ANONYMOUS MAP_ANON
#endif

/* Globals. */
unsigned char *__afl_area_ptr;
unsigned short __afl_prev_loc;

/* Map shared memory region. */
void __afl_map_shm(void) {
  char *id_str = getenv(SHM_ENV_VAR);

  if (id_str) {
    int shm_id = atoi(id_str);

    /* Store the address of the SHM region. */
    __afl_area_ptr = shmat(shm_id, NULL, 0);

  } else {

    /* If no SHM, map anyway, so we won't crash. */
    __afl_area_ptr = mmap(NULL, MAP_SIZE, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);
  }

  if (__afl_area_ptr == (void *)-1) exit(1);
}

/* Fork server logic. */
void __afl_start_forkserver() {

  /* Phone home and tell the parent that we're OK. (Note that signals with no
     SA_RESTART will mess it up). If this fails, assume that the fd is closed
     because we were execve()d from an instrumented binary, or because the
     parent doesn't want to use the fork server. */
  unsigned char tmp[4];
  if (write(FORKSRV_FD + 1, tmp, 4) != 4) return;

  while (1) {

    /* Wait for parent by reading from the pipe. Abort if read fails. */
    if (read(FORKSRV_FD, tmp, 4) != 4) exit(2);

    /* Once woken up, create a clone of our process. This is an excellent use
       case for syscall(__NR_clone, 0, CLONE_PARENT), but glibc boneheadedly
       caches getpid() results and offers no way to update the value, breaking
       abort(), raise(), and a bunch of other things :-( */
    pid_t child_pid = fork();
    if (child_pid < 0) exit(3);

    /* In child process: close fds, resume execution. */
    if (!child_pid) {
      close(FORKSRV_FD);
      close(FORKSRV_FD + 1);
      return;
    }

    /* In parent process: write PID to pipe, then wait for child. */
    if (write(FORKSRV_FD + 1, &child_pid, 4) != 4) exit(4);

    int status;
    if (waitpid(child_pid, &status, WUNTRACED) < 0) exit(5);

    /* Relay wait status to pipe, then loop back. */
    if (write(FORKSRV_FD + 1, &status, 4) != 4) exit(6);
  }
}

/* Defined in lib.rs. */
void __afl_rs_init();

/* Initialize as global ctor. */
__attribute__((constructor (0)))
void __afl_init() {
  __afl_map_shm();
  __afl_rs_init();
  __afl_start_forkserver();
}
