#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <shadow.h>
#include <stdlib.h>
#include <time.h>
#include <pwd.h>

#define MAX_STRING 2048
#define shadow_location "/home/constantin/shadow.txt"
#define passwd_location "/home/constantin/passwd.txt"
#define group_location "/home/constantin/group.txt"

// This application creates and updates users for Zentrox

// TODO 1. User update (mode = 1)
// TODO     a) Modify user
// TODO     b) Update userlist (may also have >1 users)
// TODO     c) Update .conf if required x
// TODO 2. Initial configuration update
// TODO     a) Like 1.b and .c
// TODO     b) systemctl restart

char * replace(
    char const * const original, 
    char const * const pattern, 
    char const * const replacement
) {
  size_t const replen = strlen(replacement);
  size_t const patlen = strlen(pattern);
  size_t const orilen = strlen(original);

  size_t patcnt = 0;
  const char * oriptr;
  const char * patloc;

  // find how many times the pattern occurs in the original string
  for (oriptr = original; (patloc = strstr(oriptr, pattern)); oriptr = patloc + patlen)
  {
    patcnt++;
  }

  {
    // allocate memory for the new string
    size_t const retlen = orilen + patcnt * (replen - patlen);
    char * const returned = (char *) malloc( sizeof(char) * (retlen + 1) );

    if (returned != NULL)
    {
      // copy the original string, 
      // replacing all the instances of the pattern
      char * retptr = returned;
      for (oriptr = original; (patloc = strstr(oriptr, pattern)); oriptr = patloc + patlen)
      {
        size_t const skplen = patloc - oriptr;
        // copy the section until the occurence of the pattern
        strncpy(retptr, oriptr, skplen);
        retptr += skplen;
        // copy the replacement 
        strncpy(retptr, replacement, replen);
        retptr += replen;
      }
      // copy the rest of the string.
      strcpy(retptr, oriptr);
    }
    return returned;
  }
}

int chpasswd(const char *username, const char *password)
{
    if (!strcmp("root", username))
    {
        printf("Can not change root");
        return -4;
    }    
    char *password_encrypted;
    int user_password_changed = 0;
    struct spwd *shadow_entry;
    FILE *tempfile = tmpfile();

    FILE *shadow_file = fopen(shadow_location, "r");

    if (!shadow_file)
    {
        printf("Failed to open /etc/shadow\nPlease make sure, you run this program as root.\n");
        return -2;
    }
  
    while ((shadow_entry = fgetspent(shadow_file)) != NULL)
    {
        if (!strcmp(shadow_entry->sp_namp, username))
        {
            printf("Found user with name %s \n", shadow_entry->sp_namp);
            
            char setting_prefix[MAX_STRING] = "$6$";
            char setting_suffix[2] = "$";
            char time_string[16];
            time_t current_time;
            current_time = time(NULL);
            snprintf(time_string, 16, "%ld", current_time);
            snprintf(setting_prefix, MAX_STRING - 8, "$6$%s$", time_string);
            
            password_encrypted = crypt(password, setting_prefix);

            if (password_encrypted == NULL)
            {
                printf("Failed to encrypt password.\n");
                exit(-3);
            }
             
            strncpy(shadow_entry->sp_pwdp, password_encrypted, sizeof(shadow_entry->sp_pwdp)+512);
            if (strcmp(shadow_entry->sp_pwdp, password_encrypted)) {
              printf("Failed to strncpy to struct\n");
              exit(-3);
            }

            printf("Password in struct is changed to %s\n", shadow_entry->sp_pwdp);
            
            int write_to_file = putspent(shadow_entry, tempfile);

            if (write_to_file != 0)
            {
                printf("Failed to write to file (%d)\n", write_to_file);
                exit(-4);
            }

            user_password_changed = 1;
        }
        else
        {
            putspent(shadow_entry, tempfile);
        }
    }
  
    if (user_password_changed == 0)
    {
        printf("No user was found");
        exit(-1);
    }

    rewind(tempfile);

    fclose(shadow_file);

    shadow_file = fopen(shadow_location, "w"); // ? I changed the file to a copy for now.

    int c;

    while ((c = fgetc(tempfile)) != EOF)
    {
        fputc(c, shadow_file);
    }

    fclose(tempfile);
    fclose(shadow_file);
    return 0;
}

int chusernm(const char *username, char *new_username) {
  // Define variables before user
  
  struct spwd *shadow_entry; // Shadow entry is a struct and stored in this var
  struct passwd *passwd_entry;
  
  char *passwd_line = NULL;
  char *group_line;
  char new_passwd_line[1024];
  char new_shadow_line[2048];
  char new_group_line[512];
  char new_home_folder[2048];
  char *old_home_folder;
  char c;
  
  FILE *tempfile = tmpfile(); // Tempfile for shadow
  FILE *tempfile_p = tmpfile(); // Tempfile for passwd
  FILE *tempfile_g = tmpfile(); // Tempfile for passwd
  
  FILE *shadow_file = fopen(shadow_location, "r"); // Shadow file pointer
  FILE *passwd_file = fopen(passwd_location, "r"); // Passwd file pointer
  FILE *group_file = fopen(group_location, "r");

  int change_username_shadow = 0;
  size_t passwd_line_len; 
  size_t group_line_len;
  
  if (!shadow_file)
  {
    printf("Failed to open /etc/shadow\nPlease make sure, you run this program as root.\n");
    return -2;
  }
  
  if (!passwd_file)
  {
    printf("Failed to open /etc/passwd\nPlease make sure, you run this program as root.\n");
    exit(-2);
  }

  if (!group_file)
  {
    printf("Failed to open /etc/group\nPlease make sure, you run this program as root.\n");
    exit(-2);
  }

  // Change shadow entry
  while ((shadow_entry = fgetspent(shadow_file)) != NULL) {
    if (!strcmp(shadow_entry->sp_namp, username)) {
      snprintf(new_shadow_line, sizeof(new_shadow_line) - 1, "%s:%s:%ld:%ld:%ld:%ld:%ld:%ld\n",
      new_username,
      shadow_entry->sp_pwdp,
      shadow_entry->sp_lstchg,
      shadow_entry->sp_min,
      shadow_entry->sp_max,
      shadow_entry->sp_warn,
      shadow_entry->sp_inact,
      shadow_entry->sp_expire
      );
     snprintf(new_shadow_line, sizeof(new_shadow_line) - 1, "%s", replace(new_shadow_line, ":-1", ":"));
      change_username_shadow = 1;
      fputs(new_shadow_line, tempfile);
    }
    else {
      snprintf(new_shadow_line, sizeof(new_shadow_line) - 1, "%s:%s:%ld:%ld:%ld:%ld:%ld:%ld\n",
      shadow_entry->sp_namp,
      shadow_entry->sp_pwdp,
      shadow_entry->sp_lstchg,
      shadow_entry->sp_min,
      shadow_entry->sp_max,
      shadow_entry->sp_warn,
      shadow_entry->sp_inact,
      shadow_entry->sp_expire
      );
      snprintf(new_shadow_line, sizeof(new_shadow_line) - 1, "%s", replace(new_shadow_line, ":-1", ":"));
      fputs(new_shadow_line, tempfile);
    }
  }

  if (change_username_shadow == 0) {
    printf("Failed to change username in shadow file\n");
    exit(-3);
  }
  
  // Change passwd entry 
  
  int getlineval;
  passwd_entry = getpwnam(username);
  old_home_folder = passwd_entry->pw_dir;
  snprintf(new_home_folder, sizeof(new_home_folder) - 1, "%s", replace(old_home_folder, username, new_username));
  while((getlineval = getline(&passwd_line, &passwd_line_len, passwd_file)) != -1) { 
    if (strstr(passwd_line, username)) {
      printf("Found user %s\n", username);
      snprintf(new_passwd_line, sizeof(new_passwd_line) - 1, "%s:%s:%d:%d:%s:%s:%s\n", 
      new_username,
      passwd_entry->pw_passwd,
      passwd_entry->pw_uid,
      passwd_entry->pw_gid, 
      passwd_entry->pw_gecos, 
      new_home_folder,
      passwd_entry->pw_shell);
      snprintf(new_passwd_line, sizeof(new_passwd_line) - 1, "%s", replace(new_passwd_line, ":-1", ":"));
      fputs(new_passwd_line, tempfile_p);
      printf("%s", new_passwd_line);
    }
    else {  
      fputs(passwd_line, tempfile_p);
    }
  }
  
  // Edit group file
  getlineval = 0;
  while((getlineval = getline(&group_line, &group_line_len, group_file)) != -1) { 
    if (strstr(group_line, username)) {
      printf("Found user %s\n", username);
      snprintf(new_group_line, sizeof(new_group_line) - 1, "%s", replace(group_line, username, new_username));
      fputs(new_group_line, tempfile_g);
      printf("%s", new_group_line);
    }
    else {   
      fputs(group_line, tempfile_g);
    }
  }

  // Change home folder name
  if (rename(old_home_folder, new_home_folder) != 0) {
    printf("Failed to rename user");
    exit(-3);
  }

  // Write data to files
  rewind(tempfile);
  rewind(tempfile_p);
  rewind(tempfile_g);

  fclose(shadow_file);
  fclose(passwd_file);
  fclose(group_file);
  
  shadow_file = fopen(shadow_location, "w");
  passwd_file = fopen(passwd_location, "w"); 
  group_file = fopen(group_location, "w");

  c = 0;
  while ((c = fgetc(tempfile)) != EOF)
  {
    fputc(c, shadow_file);
  }

  c = 0;
  while ((c = fgetc(tempfile_g)) != EOF)
  {
    fputc(c, group_file);
  }
  
  c = 0;
  while ((c = fgetc(tempfile_p)) != EOF)
  {
    fputc(c, passwd_file);
  }

  fclose(tempfile);
  fclose(tempfile_p);
  fclose(tempfile_g);
  
  fclose(shadow_file);
  fclose(passwd_file);
  fclose(group_file);

  return 0; 
}

int main(int argc, char *argv[])
{

    if (geteuid() == 0 || !strcmp(argv[5], "t")) {
        ;
    } else {
        printf("You are not root.\n");
        exit(-2);
    }

    int mode = 0;

    if (argc < 2)
    {
        printf("Too few arguments.\n");
        exit(-1);
    }

    if (!strcmp(argv[1], "updateUser"))
    {
        // Update user

        if (strlen(argv[4]) > 512 - 1) {
          printf("Malformed password (strlen() > 512 - 1)\n");
          exit(-1);
        }

        // Determine which attribute to change
        if (!strcmp(argv[2], "password")) {
          chpasswd(argv[3], argv[4]);
          // Change password  
        }
        else if (!strcmp(argv[2], "username")){
          printf("Username is still missing...\n");
          chusernm(argv[3], argv[4]);
        }
        else {
          printf("This user attribute is not know to this program.\n");
          exit(-1);
        }
    }
    else if (!strcmp(argv[1], "updateConfig"))
    {
        // ? Update vsftpdConfig
        printf("Config can not be edited at the time.\n");
    }
    else
    {
        printf("This command was not found.\n");
        exit(-1);
    }
}
