#include "progress-bar.h"
#include <iostream>
#include <sstream>

ProgressBar::ProgressBar(char symbol, int length, int total)
    : symbol(symbol)
    , length(length)
    , progress(0)
    , total(total)
    , startSymbol('[')
    , endSymbol(']')
    , format("{bar} {percent} {count}")
    , showPercent(false)
    , showCount(false) {}

ProgressBar& ProgressBar::update(int progress) {
    this->progress = progress;
    return *this;
}

ProgressBar& ProgressBar::showPercentage(bool show) {
    showPercent = show;
    return *this;
}

ProgressBar& ProgressBar::showCounter(bool show) {
    showCount = show;
    return *this;
}

ProgressBar& ProgressBar::setStartEndSymbols(char start, char end) {
    startSymbol = start;
    endSymbol = end;
    return *this;
}

ProgressBar& ProgressBar::setCustomFormat(const std::string& format) {
    this->format = format;
    return *this;
}

ProgressBar& ProgressBar::tick() {
    progress++;
    return *this;
}

void ProgressBar::print() const {
    std::ostringstream bar;
    std::string percent_str;
    std::string count_str;
    std::string result;

    // Generate bar component
    bar << startSymbol;
    int scaled_progress = static_cast<int>(static_cast<float>(progress) * length / total);

    for (int i = 0; i < length; i++) {
        bar << (i < scaled_progress ? symbol : ' ');
    }
    bar << endSymbol;

    // Generate percent component
    int percent = (progress * 100) / total;
    if (showPercent) {
        percent_str = std::to_string(percent) + "%";
    }

    // Generate count component
    if (showCount) {
        count_str = std::to_string(progress) + "/" + std::to_string(total);
    }

    // Process format string
    size_t pos = 0;
    std::string formatStr = format;

    // Replace placeholders with actual values
    while ((pos = formatStr.find("{bar}")) != std::string::npos) {
        formatStr.replace(pos, 5, bar.str());
    }
    while ((pos = formatStr.find("{percent}")) != std::string::npos) {
        formatStr.replace(pos, 9, percent_str);
    }
    while ((pos = formatStr.find("{count}")) != std::string::npos) {
        formatStr.replace(pos, 7, count_str);
    }

    std::cout << formatStr << (percent < 100 ? "\r" : "\n");
    std::cout.flush();
}
