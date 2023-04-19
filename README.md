# fgr

[![Rust](https://github.com/night-crawler/fgr/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/night-crawler/fgr/actions/workflows/rust.yml)
[![crates.io](https://img.shields.io/crates/v/fgr-rs.svg)](https://crates.io/crates/fgr-rs)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

Find & Grep utility with SQL-like query language.

## Examples

```bash
# Find all files with name equal to sample under the current directory:
fgr -e name=sample

# Find files with containing 's' and 777 permissions:
fgr /home /bin -e 'name=*s* and perm=777'

# Find files with name containing SAMPLE
fgr /home -e 'name="*SAMPLE*"'

# Find files with name containing SAMPLE ignore case
fgr /home -e 'name=i"*SAMPLE*"'

# Find files with name containing SAMPLE (regex)
fgr /home -e 'name=r".+SAMPLE.+"'

# Find files with name containing SAMPLE ignore case (regex)
fgr /home -e 'name=ri".+SAMPLE.+"'

# Find files under the /bin directory not owned by root:
fgr /bin -e 'user > 0'

# Find files under the /bin directory having suid bit (but not limited to):
fgr /bin -e 'perms>4000'

# Find recently accessed files (but not in future):
fgr /home -e 'atime > now - 1h and atime < now'

# Find stuff in files:
fgr /home -e 'type=text and contains=*stuff*'

# Other examples:
fgr /home /bin -e 'name=*s* and perm=777 or (name=*rs and contains=r".+user.is_birthday.*")'
fgr /home /bin -e 'name=*s* and perm=777 or (name=*rs and contains=*birth*)'
fgr /home /bin -e 'ext=so and mtime >= now - 1d'
fgr /home -e 'size>=1Mb and name != *.rs and type=vid'

# xargs & -print0 support
fgr /home -e 'perms=777' -p | xargs -0 -n1 | sort

```

## Features
 
 - Filter files by:
   - Size
   - Depth
   - Type (text, app, archive, audio, book, doc, font, img, vid)
   - atime, mtime
   - name, extension
   - contents
   - user, group, permissions
 - Timeout IO operations (does not hang parsing files like `/sys/kernel/security/apparmor/revision`) 
 - Regex & Glob name matching
 - Regex & Glob contents matching
 - Nexted expressions
 - Human-readable atime/mtime search patterns
 - `.ignore` support, thanks to [ignore](https://docs.rs/ignore/latest/ignore/) crate

## Speed

By default, it acts like the `find` and visits all directories.
Search by name is quite fast

```bash
du -h /home
# 98G     /home
# About 100G of .gradle, caches and all the whatnot

sudo sh -c 'echo 1 >/proc/sys/vm/drop_caches'
sudo sh -c 'echo 2 >/proc/sys/vm/drop_caches'
sudo sh -c 'echo 3 >/proc/sys/vm/drop_caches'

fgr /home -e 'name=*sample*' # 1.09s user 2.70s system 169% cpu 2.239 total

sudo sh -c 'echo 1 >/proc/sys/vm/drop_caches'
sudo sh -c 'echo 2 >/proc/sys/vm/drop_caches'
sudo sh -c 'echo 3 >/proc/sys/vm/drop_caches'

find /home -name '*sample*' # 0.71s user 2.09s system 12% cpu 22.156 total
```

## TODO

- [x] Query precedence evaluation
- [ ] Run command
- [ ] Exclude patterns & default exclude patterns (handling `/proc/**/pagemap` scenarios)
- [x] Binary/Text type detector
- [x] Ignore case searches
- [ ] Error printing
- [ ] Documentation
- [ ] AUR
