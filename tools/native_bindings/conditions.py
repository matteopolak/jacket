"""Boolean configuration coverage over the requested desktop/mobile targets."""
import ast
from functools import lru_cache
from itertools import product
import re


@lru_cache(None)
def expression(condition):
    code = condition.replace('&&',' and ').replace('||',' or ')
    code = re.sub(r'!(?!=)',' not ',code)
    code = re.sub(r'\.([A-Z][A-Z_0-9]*)',lambda m:repr(m[1]),code)
    code = re.sub(r'\btrue\b','True',code)
    code = re.sub(r'\bfalse\b','False',code)
    tree = ast.parse(code.strip() or 'True',mode='eval')
    permitted = (ast.Expression,ast.BoolOp,ast.UnaryOp,ast.Compare,ast.Constant,
                 ast.And,ast.Or,ast.Not,ast.Eq,ast.NotEq,ast.Name,ast.Load)
    if any(not isinstance(node,permitted) for node in ast.walk(tree)):
        raise ValueError('Unsupported condition: '+condition)
    names = {node.id for node in ast.walk(tree) if isinstance(node,ast.Name)}
    return compile(tree,'<reviewed-condition>','eval'),frozenset(names)


def evaluate(condition, values):
    code, names = expression(condition)
    missing = names-values.keys()
    if missing:
        raise ValueError('Unknown configuration values: '+', '.join(sorted(missing)))
    return bool(eval(code,{'__builtins__':{}},values))


@lru_cache(None)
def possible(condition):
    _, names = expression(condition)
    flags = sorted(names-{'OS','CPU'})
    for system,cpu in product(('WINDOWS','LINUX','MACOS','ANDROID','IOS'),('X64','ARM64')):
        for states in product((False,True),repeat=len(flags)):
            if evaluate(condition,dict(zip(flags,states),OS=system,CPU=cpu)):
                return True
    return False
