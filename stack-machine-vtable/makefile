CFLAGS := -Wall -Wextra -pedantic -Wconversion -Wsign-conversion -g
PROG := stk

$(PROG): main.o stack.o value.o result.o bytes.o
	$(CC) $(CFLAGS) $^ -o $(PROG)

clean:
	rm -r *.o $(PROG)
