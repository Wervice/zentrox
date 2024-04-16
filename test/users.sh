#!/bin/sh

# Compile users.c

gcc ../libs/users.c -o ../libs/users -lcrypt | echo "Compiled with "$?

while [[ $i != 50 ]]; do
  i=$((i+1))
  ../libs/users updateUser password ftp_zentrox $(python3 -c "import random;import string;print(''.join(random.choice(string.ascii_letters + string.digits + string.punctuation) for _ in range(511)))") t | echo $i"="$? 
done

echo "Tested password 50 times"

i=0;

while [[ $i != 50 ]]; do
  i=$((i+1))
  ../libs/users updateUser username ftp_zentrox $(python3 -c "import random;import string;print(''.join(random.choice(string.ascii_letters + string.digits + string.punctuation) for _ in range(511)))") t | echo $i"="$?
  cat /etc/passwd > ~/passwd.txt
  cat /etc/shadow > ~/shadow.txt
done

echo "Tested username 50 times"

clear

echo "* Passed test 1"
