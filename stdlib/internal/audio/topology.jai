// The public topology arrays intentionally remain live mutable storage.
arcs_6ch := Vbap_Arc.[
    .{ .CENTER, .FRONT_LEFT, .{} },
    .{ .FRONT_LEFT, .REAR_LEFT, .{} },
    .{ .REAR_LEFT, .REAR_RIGHT, .{} },
    .{ .REAR_RIGHT, .FRONT_RIGHT, .{} },
    .{ .FRONT_RIGHT, .CENTER, .{} }
];

arcs_8ch := Vbap_Arc.[
    .{ .CENTER, .FRONT_LEFT, .{} },
    .{ .FRONT_LEFT, .REAR_LEFT, .{} },
    .{ .REAR_LEFT, .EXTEND_LEFT, .{} },
    .{ .EXTEND_LEFT, .EXTEND_RIGHT, .{} },
    .{ .EXTEND_RIGHT, .REAR_RIGHT, .{} },
    .{ .REAR_RIGHT, .FRONT_RIGHT, .{} },
    .{ .FRONT_RIGHT, .CENTER, .{} }
];

arcs: [] Vbap_Arc;
