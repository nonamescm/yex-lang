#include "yex.h"
#include <stddef.h>
#include <stdlib.h>
#include <string.h>
#define IS_NULL(t) (t) == NULL
yex_num_ptr_t yex_num(double num)
{
  yex_num_ptr_t ptr = malloc(sizeof(yex_num_t));
  if (IS_NULL(ptr)) return NULL;
  else {
    memcpy(ptr, &num, sizeof(yex_num_t));
    return ptr;
  }
}
yex_string_t yex_init_str(const char* content) 
{
  yex_string_t ptr = malloc(strlen(content));
  if (IS_NULL(ptr)) return NULL;
  else {
    strncpy(ptr, content, strlen(content));
    return ptr;
  }
}
