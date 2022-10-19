# fgr

Find & Grep utility with SQL-like query language.

## Examples

```bash

# Find all files with name equal to sample under the current directory:
fgr -e name=sample

# Find files with containing 's' and 777 permissions:
fgr /home /bin -e 'name=*s* and perm=777'

# Find files under the /bin directory not owned by root:
fgr /bin -e 'user > 0'

# Find files under the /bin directory having suid bit (but not limited to):
fgr /bin -e 'perms>4000'

# Find recently accessed files (but not in future):
fgr /home -e 'atime > now - 1h and atime < now - 0h'

# Find stuff in files:
fgr /home -e 'type=text and contains=*stuff*'

# Other examples:
fgr /home /bin -e 'name=*s* and perm=777 or (name=*rs and contains=r".+user.is_birthday.*")'
fgr /home /bin -e 'name=*s* and perm=777 or (name=*rs and contains=*birth*)'
fgr /home /bin -e 'ext=so and mtime >= now - 1d'
fgr /home -e 'size>=1Mb and name != *.rs and type=vid'
```