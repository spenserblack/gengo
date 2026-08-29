# `fib n` is the n-th Fibonacci number
export def fib [n: int]: nothing -> int {
    if $n == 0 {
        return 0
    } else if $n == 1 {
        return 1
    }

    (fib ($n - 1)) + (fib ($n - 2))
}
