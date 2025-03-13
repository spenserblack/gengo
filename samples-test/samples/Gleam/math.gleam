//// Simple math demonstration on Gleam

import gleam/io

// Main function
pub fn main() {
  // Test Fibonacci function
  io.debug(fib(10))
  // Test add function
  io.debug(add(first: 1, second: -999))
  // Test abs function
  io.debug(abs(-1))
  io.debug(abs(91_837_524))
  // Test min and max functions
  io.debug(min(140, 0))
  io.debug(min(0, 0))
  io.debug(max(-1, 0))
  // Test is_even function
  io.debug(is_even(0))
  io.debug(is_even(10))
  // Test clamp function
  io.debug(clamp(10, min: 11, max: 20))
}

/// Get Fibonacci for given number
pub fn fib(n: Int) {
  case n {
    _ if n < 0 -> panic as "n should not be negative!"
    0 -> 0
    1 -> 1
    _ -> fib(n - 1) + fib(n - 2)
  }
}

/// Add one number to other
pub fn add(first num1: Int, second num2: Int) -> Int {
  num1 + num2
}

/// Get absolute number of value
pub fn abs(x: Int) -> Int {
  case x >= 0 {
    True -> x
    False -> x * -1
  }
}

/// Return min value of given pair
pub fn min(a: Int, b: Int) -> Int {
  case a < b {
    True -> a
    False -> b
  }
}

/// Return max value of given pair
pub fn max(a: Int, b: Int) -> Int {
  case a > b {
    True -> a
    False -> b
  }
}

/// Returns true if given number is even and false if it's odd
pub fn is_even(x: Int) -> Bool {
  x % 2 == 0
}

/// Restricts an int between a lower and upper bound.
pub fn clamp(x: Int, min min_bound: Int, max max_bound: Int) -> Int {
  x
  |> min(max_bound)
  |> max(min_bound)
}

fn todo_func() {
  todo as "Not implemented yet"
}

fn panic_func() {
  panic as "VERY BAD ERROR!!!"
}
