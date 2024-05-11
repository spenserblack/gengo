;; Copy contents of one file to another

(require 'uiop)

(let* ((args (uiop:command-line-arguments))
       (in_file (first args))
       (out_file (second args)))
  (with-open-file (in-stream in_file)
    (with-open-file (out-stream out_file :direction :output :if-exists :supersede :if-does-not-exist :create)
      (loop for line = (read-line in-stream nil)
            while line
            do (write-line line out-stream)))))
