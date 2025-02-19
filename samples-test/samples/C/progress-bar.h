#include <stdio.h>
#include <stdbool.h>
#include <string.h>

typedef struct {
    char symbol;
    char startSymbol;
    char endSymbol;
    int length;
    int progress;
    int total;
    char* format;
    bool showPercent;
    bool showCount;
} ProgressBar;

ProgressBar init(char symbol, int length, int total);
ProgressBar update(ProgressBar *pb, int progress);
ProgressBar showPercent(ProgressBar *pb, bool show);
ProgressBar showCount(ProgressBar *pb, bool show);
ProgressBar setStartEndSymbols(ProgressBar *pb, char start, char end);
ProgressBar setCustomFormat(ProgressBar *pb, char* format);
ProgressBar tick(ProgressBar *pb);
void print(ProgressBar pb);
