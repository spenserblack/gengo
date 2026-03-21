// Define a recursive enum for expressions
enum Expr { Num(Int); Mul(Expr, Expr) }

// Evaluate expression with pattern matching
fn eval(expr : Expr) -> Int {
  match expr {
    Num(n) => n
    Mul(a, b) => eval(a) * eval(b)
  }
}

fn main {
  let expr = Mul(Mul(Num(2), Num(3)), Num(4))
  println(eval(expr))  // 24
}