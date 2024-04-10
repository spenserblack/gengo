; Provides basic mathematical utilities

; Gets n!
(define (factorial n)
  (if (<= n 1)
    1
    (* n (factorial (- n 1)))))

; Gets the nth element of the fibonacci sequence
(define (fibonacci n)
  (cond ((< n 0) (error "n cannot be negative"))
        ((<= n 1) n)
        (else (+ (fibonacci (- n 1)) (fibonacci (- n 2))))))

; Reverses a list
(define (reverse-list l)
  (if (null? l)
    '()
    (append (reverse-list (cdr l)) (list (car l)))))
