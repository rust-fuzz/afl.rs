/*

   A simple proof-of-concept for instrumented strcpy() or memcpy().

   Normally, afl-fuzz will have difficulty ever reaching the code behind
   something like:

     if (!strcmp(password, "s3cr3t!")) ...

   This is because the strcmp() operation is completely opaque to the tool.
   A simple and non-invasive workaround that doesn't require complex code
   analysis is to replace strcpy(), memcpy(), and equivalents with
   inlined, non-optimized code.

   I am still evaluating the value of doing this, but for time being, here's
   a quick demo of how it may work. To test:

     $ ./afl-gcc instrumented_cmp.c
     $ mkdir test_in
     $ printf xxxxxxxxxxxxxxxx >test_in/input
     $ ./afl-fuzz -i test_in -o test_out ./a.out

 */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

/* Naive instrumented memcmp(). */

inline int my_memcmp(char* ptr1, char* ptr2, int len)
  __attribute__((always_inline));

inline int my_memcmp(char* ptr1, char* ptr2, int len) {

  while (len--) if (*(ptr1++) ^ *(ptr2++)) return 1;
  return 0;

}

#define memcmp my_memcmp

/* Normal program. */

char tmp[16];

int main(int argc, char** argv) {

  int len = read(0, tmp, sizeof(tmp));

  if (len != sizeof(tmp)) {

    printf("Truncated file!\n");
    exit(1);

  }

  if (!memcmp(tmp + 5, "sesame", 6)) {

    /* Simulated "faulty" code path. */

    int* x = (int*)0x12345678;
    *x = 1;

  } else printf("Bad password.\n");

  return 0;

}
