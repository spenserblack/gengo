#include "progress-bar.h"
#include <stdio.h>
#include <stdbool.h>
#include <string.h>

ProgressBar init(char symbol, int length, int total) {
    ProgressBar pb;
    pb.symbol = symbol;
    pb.length = length;
    pb.progress = 0;
    pb.showPercent = false;
    pb.total = total;
    pb.startSymbol = '[';
    pb.endSymbol = ']';
    pb.format = "{bar} {percent} {count}";
    return pb;
}

ProgressBar update(ProgressBar *pb, int progress) {
    pb->progress = progress;
    return *pb;
}

ProgressBar showPercent(ProgressBar *pb, bool show) {
    pb->showPercent = show;
    return *pb;
}

ProgressBar showCount(ProgressBar *pb, bool show) {
    pb->showCount = show;
    return *pb;
}

ProgressBar setStartEndSymbols(ProgressBar *pb, char start, char end) {
    pb->startSymbol = start;
    pb->endSymbol = end;
    return *pb;
}

ProgressBar setCustomFormat(ProgressBar *pb, char* format) {
    pb->format = format;
    return *pb;
}

ProgressBar tick(ProgressBar *pb) {
    pb->progress++;
    return *pb;
}

void print(ProgressBar pb) {
    char bar[256] = "";
    char percent_str[32] = "";
    char count_str[32] = "";
    char result[512] = "";
    char *format = pb.format;
    
    // Generate bar component
    sprintf(bar, "%c", pb.startSymbol);
    int scaled_progress = (int)((float)pb.progress * pb.length / pb.total);
    
    for (int i = 0; i < pb.length; i++) {
        if (i < scaled_progress) {
            sprintf(bar + strlen(bar), "%c", pb.symbol);
        } else {
            strcat(bar, " ");
        }
    }
    sprintf(bar + strlen(bar), "%c", pb.endSymbol);
    
    // Generate percent component
    int percent = (pb.progress * 100) / pb.total;
    if (pb.showPercent) {
        sprintf(percent_str, "%d%%", percent);
    }
    
    // Generate count component
    if (pb.showCount) {
        sprintf(count_str, "%d/%d", pb.progress, pb.total);
    }
    
    // Process format string
    char *ptr = format;
    while (*ptr) {
        if (*ptr == '{') {
            if (strncmp(ptr, "{bar}", 5) == 0) {
                strcat(result, bar);
                ptr += 5;
            } else if (strncmp(ptr, "{percent}", 9) == 0) {
                if (pb.showPercent) {
                    strcat(result, percent_str);
                }
                ptr += 9;
            } else if (strncmp(ptr, "{count}", 7) == 0) {
                if (pb.showCount) {
                    strcat(result, count_str);
                }
                ptr += 7;
            } else {
                strncat(result, ptr, 1);
                ptr++;
            }
        } else {
            strncat(result, ptr, 1);
            ptr++;
        }
    }
    
    printf("%s", result);
    
    if (percent < 100) {
        printf("\r");
    } else {
        printf("\n");
    }
}
