#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

int main(int argc, char *argv[]) {
  char *password = argv[1];
  char time_string[16];
  char *password_encrypted;
  char setting_prefix[1024];
  time_t current_time;

  current_time = time(NULL);
  snprintf(time_string, 16, "%ld", current_time);
  snprintf(setting_prefix, 1016, "$6$%s$", time_string);     
  password_encrypted = crypt(password, setting_prefix);
  printf("%s", password_encrypted);
}
