// Route templates and the router. A template such as `/users/{id}/posts/{post}` or
// `/files/{path...}` is parsed at compile time (by `get` and friends, which check the handler
// against it) and again at startup, when the routes are put into the router.
//
// The router is a tree with one level per path segment (a radix tree whose edges are whole
// segments, which is what paths split on anyway). At each level a static segment wins over a
// `{param}`, which wins over a `{rest...}` tail; a branch that fails further down is backed out
// of, so `/users/new` and `/users/{id}/posts` coexist.

#scope_module

Segment_Kind :: enum u8 {
    STATIC;
    PARAM;
    REST;
}

Segment :: struct {
    kind: Segment_Kind;
    text: string; // the literal text, or the parameter's name
}

Template :: struct {
    segments: [..] Segment;
    error: string; // empty when the template is fine
}

// Splits `/users/{id}` into segments. A trailing `/` is ignored (`/users/` is `/users`). Names are
// identifiers; `{name...}` takes the rest of the path and may only be last; a `{param}` must be
// a whole segment. Strings point into `template`.
parse_template :: (template: string) -> Template {
    result: Template;

    if template.count == 0 || template[0] != #char "/" {
        result.error = "a route template must start with `/`";

        return result;
    }

    rest := slice(template, 1, template.count - 1);

    if rest.count > 0 && rest[rest.count - 1] == #char "/" rest.count -= 1;

    if rest.count == 0 return result;

    start := 0;

    while start <= rest.count {
        end := start;

        while end < rest.count && rest[end] != #char "/" end += 1;

        piece := slice(rest, start, end - start);

        if piece.count == 0 {
            result.error = "a route template has an empty segment (`//`)";

            return result;
        }

        segment: Segment;

        brace_open := find_index_from_left(piece, #char "{");
        brace_close := find_index_from_left(piece, #char "}");

        if brace_open < 0 && brace_close < 0 {
            segment.kind = .STATIC;
            segment.text = piece;
        } else if brace_open != 0 || brace_close != piece.count - 1 {
            result.error = tprint("`%` is not a valid segment: a {param} must be a whole segment, as in `/users/{id}`", piece);

            return result;
        } else {
            name := slice(piece, 1, piece.count - 2);

            segment.kind = .PARAM;

            if name.count > 3 && ends_with(name, "...") {
                name = slice(name, 0, name.count - 3);
                segment.kind = .REST;
            }

            if !is_identifier(name) {
                result.error = tprint("`%` is not a valid parameter name: use letters, digits and `_`, not starting with a digit", piece);

                return result;
            }

            for existing: result.segments {
                if existing.kind != .STATIC && existing.text == name {
                    result.error = tprint("the parameter `{%}` appears twice", name);

                    return result;
                }
            }

            segment.text = name;
        }

        if segment.kind == .REST && end < rest.count {
            result.error = tprint("`{%...}` takes the rest of the path, so it must be the last segment", segment.text);

            return result;
        }

        array_add(*result.segments, segment);

        start = end + 1;
    }

    return result;
}

is_identifier :: (name: string) -> bool {
    if name.count == 0 return false;

    for name {
        letter := (it >= #char "a" && it <= #char "z") || (it >= #char "A" && it <= #char "Z") || it == #char "_";
        digit := it >= #char "0" && it <= #char "9";

        if !letter && !(digit && it_index > 0) return false;
    }

    return true;
}

// The names of a template's `{params}` in order, for the compile-time checks.
template_names :: (template: Template) -> [..] string {
    names: [..] string;

    for template.segments {
        if it.kind != .STATIC array_add(*names, it.text);
    }

    return names;
}

// ---------------------------------------------------------------------------------------------
// The router

Route :: struct {
    method: Method;
    template: string;
    handler: Handler;
    middleware: [] Middleware;
    names: [..] string;
}

Static_Child :: struct {
    text: string;
    node: *Node;
}

Node :: struct {
    statics: [..] Static_Child;
    param: *Node;
    rest: *Node;
    routes: [..] *Route;
}

Router :: struct {
    root: Node;
    routes: [..] *Route;
}

// What `router_match` found.
Match :: struct {
    route: *Route;
    // A node that has the path but no route for the method: its methods go into `Allow` (405).
    path_node: *Node;
    values: [..] string; // the {param} values, raw
}

// Puts `route` into the router. Returns an error message when it conflicts with a route already
// there (same method, same shape: parameter names do not matter).
router_add :: (router: *Router, route: *Route) -> string {
    template := parse_template(route.template);

    defer array_reset(*template.segments);

    if template.error.count return tprint("route %: %", route.template, template.error);

    node := *router.root;

    for template.segments {
        if it.kind == .STATIC {
            found: *Node;

            for child: node.statics {
                if child.text == it.text found = child.node;
            }

            if !found {
                found = New(Node);
                array_add(*node.statics, Static_Child.{ it.text, found });
            }

            node = found;
        } else if it.kind == .PARAM {
            if !node.param node.param = New(Node);

            node = node.param;
        } else {
            if !node.rest node.rest = New(Node);

            node = node.rest;
        }
    }

    for node.routes {
        if it.method == route.method {
            return tprint("routes conflict: % % and % % match the same requests", method_name(it.method), it.template, method_name(route.method), route.template);
        }
    }

    for template.segments {
        if it.kind != .STATIC array_add(*route.names, it.text);
    }

    array_add(*node.routes, route);
    array_add(*router.routes, route);

    return "";
}

router_free :: (router: *Router) {
    free_node_children(*router.root);

    for router.routes {
        array_reset(*it.names);
        array_free(it.middleware);
        free(it);
    }

    array_reset(*router.routes);
    array_reset(*router.root.routes);
    array_reset(*router.root.statics);
    router.root = .{};
} @Releases

free_node_children :: (node: *Node) {
    for node.statics {
        free_node_children(it.node);
        array_reset(*it.node.routes);
        array_reset(*it.node.statics);
        free(it.node);
    }

    if node.param {
        free_node_children(node.param);
        array_reset(*node.param.routes);
        array_reset(*node.param.statics);
        free(node.param);
    }

    if node.rest {
        array_reset(*node.rest.routes);
        free(node.rest);
    }
} @Releases

// The route of `node` for `method`. A HEAD request is served by the GET route when there is no
// HEAD one.
route_for_method :: (node: *Node, method: Method) -> *Route {
    for node.routes if it.method == method return it;

    if method == .HEAD {
        for node.routes if it.method == .GET return it;
    }

    return null;
}

// Finds the route for a request path. `segments` are the percent-decoded segments of the path and
// `raw_path` the path as sent (the `{rest...}` value is cut from it).
router_match :: (router: *Router, segments: [] string, raw_path: string, method: Method) -> Match {
    match: Match;

    search(*router.root, segments, 0, raw_path, method, *match);

    return match;
}

search :: (node: *Node, segments: [] string, index: int, raw_path: string, method: Method, match: *Match) -> bool {
    if index == segments.count {
        if node.routes.count > 0 {
            route := route_for_method(node, method);

            if route {
                match.route = route;

                return true;
            }

            if !match.path_node match.path_node = node;
        }

        return false;
    }

    segment := segments[index];

    for child: node.statics {
        if child.text == segment {
            mark := match.values.count;

            if search(child.node, segments, index + 1, raw_path, method, match) return true;

            match.values.count = mark;
        }
    }

    if node.param && segment.count > 0 {
        mark := match.values.count;

        array_add(*match.values, segment);

        if search(node.param, segments, index + 1, raw_path, method, match) return true;

        match.values.count = mark;
    }

    if node.rest && segment.count > 0 {
        route := route_for_method(node.rest, method);

        if route {
            // The value is the path from this segment on, as sent.
            offset := 0;
            skipped := 0;

            while offset < raw_path.count && skipped <= index {
                if raw_path[offset] == #char "/" skipped += 1;

                offset += 1;
            }

            array_add(*match.values, percent_decode(slice(raw_path, offset, raw_path.count - offset),, temp));

            match.route = route;

            return true;
        }

        if !match.path_node && node.rest.routes.count > 0 match.path_node = node.rest;
    }

    return false;
}

// Splits a request path into segments. `//` gives an empty segment (which no route matches), a
// trailing `/` is ignored.
split_path :: (path: string) -> [..] string {
    segments: [..] string;

    if path.count <= 1 return segments;

    rest := slice(path, 1, path.count - 1);

    if rest.count > 0 && rest[rest.count - 1] == #char "/" rest.count -= 1;

    if rest.count == 0 return segments;

    start := 0;

    while start <= rest.count {
        end := start;

        while end < rest.count && rest[end] != #char "/" end += 1;

        array_add(*segments, slice(rest, start, end - start));

        start = end + 1;
    }

    return segments;
}

// `GET, HEAD, POST` for a 405 or an automatic OPTIONS answer.
allowed_methods :: (node: *Node) -> string {
    order :: Method.[.GET, .HEAD, .POST, .PUT, .PATCH, .DELETE, .OPTIONS];

    builder: String_Builder;
    first := true;

    for method: order {
        listed := false;

        for node.routes {
            if it.method == method listed = true;
        }

        if method == .HEAD {
            for node.routes if it.method == .GET listed = true;
        }

        if method == .OPTIONS listed = true;

        if !listed continue;

        if !first append(*builder, ", ");

        first = false;

        append(*builder, method_name(method));
    }

    return builder_to_string(*builder,, temp);
}
