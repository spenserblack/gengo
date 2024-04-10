SELECT employee.id
        , employee.name
        , job.title AS job_title
    FROM employee
    LEFT JOIN job ON employee.id = job.employee_id
    WHERE employee.id = @id
