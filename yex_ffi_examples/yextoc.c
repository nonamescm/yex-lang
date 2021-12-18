#include "yex/yex.h"
#include <stdio.h>
#include <string.h>
#include <stdlib.h>
unsigned int addr_of(void* ptr) {
  return *(unsigned int*) ptr;
}
void c_args_example(int _argc, yex_args_t args) {
  yex_string_ptr_t arg0 = yex_get_str(args[0]);
  yex_num_ptr_t    arg1 = args[1];
  printf("I Received Arg number(%d): %s at %x\n", 0, *arg0, addr_of(arg0));
  printf("I Received Arg number(%d): %f at %x\n", 1, *arg1, addr_of(&arg1));
}
yex_num_ptr_t c_sum(int _argc, yex_args_t args) {
    yex_num_ptr_t a = args[0];
    yex_num_ptr_t b = args[1];
    return yex_num(*a + *b);
}
void c_reads(int _argc, yex_args_t args) {
    yex_string_ptr_t s = yex_get_str(args[0]);
    printf("I Get A String From Yex: %s at %x with len %lu\n", *s, addr_of(s), strlen(*s));
}