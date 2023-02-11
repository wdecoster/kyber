# kyber

Tool to quickly make a minimalistic 300x300 pixels image of read length (log transformed) and read accuracy.

## DETAILS

Both the x and y axis are fixed, allowing for comparison across datasets. The current settings should work for most (long-read) datasets, let me know if you disagree.
The x-axis has log transformed read lengths, with a maximum length of 1M. The 'major' ticks on the axis are at 10, 100, 1000 and 100kb.
The y-axis has the gap-compressed reference identity, ranging from 70% to 100% with a major tick at 80% and 90%. When using Phred-scaled accuracy scores, the y-axis ranges from Q0 to Q40, with major ticks at Q10, Q20, Q30.

## EXAMPLE

Gap compressed percent identity             |  Phred-scale accuracy
:-------------------------:|:-------------------------:
![example](example/accuracy_heatmap.png)  |  ![example](example/accuracy_heatmap_phred.png)

## INSTALLATION

## USAGE

```text
Usage: kyber [OPTIONS] <INPUT>

Arguments:
  <INPUT>  cram or bam file, or use `-` to read from stdin

Options:
  -t, --threads <THREADS>  Number of parallel decompression threads to use [default: 4]
  -o, --output <OUTPUT>    Output file name [default: accuracy_heatmap.png]
  -c, --color <COLOR>      Color used for heatmap [default: green] [possible values: red, green, blue, purple, yellow]
  -p, --phred              Plot accuracy in phred scale
  -h, --help               Print help
  -V, --version            Print version
```
