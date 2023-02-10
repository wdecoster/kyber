# kyber

Tool to quickly make a minimalistic 300x300 pixels image of read length (log transformed) and read accuracy.

## DETAILS

Both the x and y axis are fixed, allowing for comparison across datasets. The current settings should work for most (long-read) datasets, let me know if you disagree.
The x-axis has log transformed read lengths, with a maximum length of 1M. The 'major' ticks on the axis are at 10, 100, 1000 and 100kb.
The y-axis has the gap-compressed reference identity, ranging from 70% to 100% with a major tick at 80% and 90%.

![example](example/accuracy_heatmap.png)

## USAGE

```text
Usage: kyber [OPTIONS] <INPUT>

Arguments:
  <INPUT>  cram or bam file to create plot from

Options:
  -t, --threads <THREADS>  Number of parallel decompression threads to use [default: 4]
  -o, --output <OUTPUT>    Output file name [default: accuracy_heatmap.png]
  -c, --color <COLOR>      Color used for heatmap [default: green] [possible values: red, green, blue, purple, yellow]
  -h, --help               Print help
  -V, --version            Print version
```
