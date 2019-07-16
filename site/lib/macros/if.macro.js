const { createMacro, MacroError } = require('babel-plugin-macros');
const t = require('@babel/types');

module.exports = createMacro($if);

function $if({ references }) {
    references.default.forEach(path => {
        let cond = path.parentPath;
        let then = cond.parentPath.parentPath;
        let el = then.parentPath.parentPath;

        if (!(t.isCallExpression(cond.node) && t.isCallExpression(then.node) && t.isCallExpression(el.node))) {
            throw new MacroError("Illegal use of if macro, see usage.");
        }

        if (!(t.isIdentifier(cond.node.callee) && t.isMemberExpression(then.node.callee) && t.isMemberExpression(el.node.callee))) {
            throw new MacroError("Illegal use of if macro, see usage.");
        }

        if (!(then.node.callee.property.name === "then" && el.node.callee.property.name === "else")) {
            throw new MacroError("Illegal use of if macro, see usage.");
        }

        if (cond.node.arguments.length !== 1) {
            throw new MacroError("Arguments to if macro must contain only 1 argument.");
        }

        if (then.node.arguments.length !== 1) {
            throw new MacroError("Arguments to then macro must contain only 1 argument.");
        }

        if (el.node.arguments.length !== 1) {
            throw new MacroError("Arguments to else macro must contain only 1 argument.");
        }

        el.replaceWith(t.conditionalExpression(cond.node.arguments[0], then.node.arguments[0], el.node.arguments[0]))
    });
}
