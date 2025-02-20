/**
 * Parses and evaluates a mathematical expression string
 * @param {string} expression - The mathematical expression to evaluate
 * @returns {number} The result of the expression
 */
function parseExpression(expression) {
    try {
        // Remove whitespace and validate input
        const cleanExpression = expression.replace(/\s+/g, '');
        
        // Check for invalid characters
        if (/[^0-9+\-*/().]/.test(cleanExpression)) {
            throw new Error('Invalid characters in expression');
        }

        // Use Function constructor to safely evaluate the expression
        // This handles basic arithmetic operations: +, -, *, /
        return Function(`return ${cleanExpression}`)();
    } catch (error) {
        throw new Error('Invalid expression: ' + error.message);
    }
}

module.exports = parseExpression;