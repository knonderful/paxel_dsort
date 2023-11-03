# dicksort
Rust learning project. Sorts your fav pics into a directory structure

out of the box usage:
```bash
dick_sort <source> <destination>
```
will copy all images with an exif date into a dir yyyy/mm/dd/ in destination

```bash
dick_sort -m <source> <destination>
```
will move them instead

```bash
dick_sort -m -c <source> <destination>
```
will move them and also delete source dirs that are empty after the move;
including the `<destination>` if all was moved.


```bash
dick_sort -d <source> <destination>
```
will only describe which files it would copy / move

```bash
dick_sort -r <source> <destination>
```
will do it recursively

# plans

* add the ability to define the output path e.g. `--format "$year/$month/$day"`
* add the ability to parse and use tags from the filename e.g. --format "$person/$year/$location" --parse "person=.*/([a-z]).*Jpg” ...
* release binaries
* extract location maybe?


