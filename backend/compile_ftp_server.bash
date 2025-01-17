cython --embed ftp.py &&
	gcc $(pkg-config --cflags python3) $(python-config --ldflags --embed) ftp.c -o ftp &&
	sudo setcap 'cap_net_bind_service=+ep' ./ftp &&
	ldd ./ftp
