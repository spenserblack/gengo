#ifndef PROGRESS_BAR_H
#define PROGRESS_BAR_H

#include <string>

class ProgressBar {
private:
    char symbol;
    char startSymbol;
    char endSymbol;
    int length;
    int progress;
    int total;
    std::string format;
    bool showPercent;
    bool showCount;

public:
    ProgressBar(char symbol, int length, int total);

    ProgressBar& update(int progress);
    ProgressBar& showPercentage(bool show);
    ProgressBar& showCounter(bool show);
    ProgressBar& setStartEndSymbols(char start, char end);
    ProgressBar& setCustomFormat(const std::string& format);
    ProgressBar& tick();
    void print() const;
};

#endif
