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



```bash
dick_sort -f "[YEAR]_[MONTH]/[DATE]_" <source> <destination>
```
will replace the default target path from <destination>/year/month/date/filename to <destination>/year_month/date_filename. In case a flatter hierarchy is wanted.
The year will always have 4, the month 2 and the day also 2 numbers e.g. <destination>/2023_11/05_my_little_pony.jpeg

# plans

* add the ability to parse and use tags from the filename e.g. --format "$person/$year/$location" --parse "person=.*/([a-z]).*Jpg” ...
* release binaries
* extract location maybe?


