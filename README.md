# dicksort
Rust learning project. Sorts your fav pics into a directory structure

out of the box usage:
```bash
dicksort <source> <destination>
```
will copy all images with exif create date into a dir yyyy/mm/dd/ in destination

```bash
dicksort -m <source> <destination>
```
will move them instead

```bash
dicksort -d <source> <destination>
```
will only describe which files it would copy / move

```bash
dicksort -r <source> <destination>
```
will do it recursively

