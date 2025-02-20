import React from 'react';

const Button = ({ onClick, children, disabled = false, className = '' }) => {
    return (
        <button
            onClick={onClick}
            disabled={disabled}
            className={`button ${className}`}
        >
            {children}
        </button>
    );
};

export default Button;