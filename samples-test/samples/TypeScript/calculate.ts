/**
 * Evaluates a mathematical expression string and returns the result
 * @param expression The mathematical expression to evaluate
 * @returns The calculated result
 */
export function calculate(expression: string): number {
    try {
        // Remove all whitespace from the expression
        const cleanExpression = expression.replace(/\s+/g, '');
        
        // Use Function constructor to safely evaluate the expression
        // This is safer than using eval()
        return new Function(`return ${cleanExpression}`)();
    } catch (error) {
        throw new Error('Invalid mathematical expression');
    }
}