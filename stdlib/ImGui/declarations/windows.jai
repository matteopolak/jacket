// Generated from docs/specs/stdlib/modules/imgui.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

// The declarations for `OS == .WINDOWS`, loaded by declarations.jai.

#scope_file
#import "Math";

#if OS == .MACOS {
    #library,system,link_always "c++"; @native(external)
} else #if OS == .LINUX {
    #library,system,link_always "stdc++"; @native(external)
}

imgui :: #library,system "imgui";
#scope_export

BackendFlags :: enum_flags s32 {
    None :: 0x0;
    HasGamepad :: 0x1;
    HasMouseCursors :: 0x2;
    HasSetMousePos :: 0x4;
    RendererHasVtxOffset :: 0x8;
    ImGuiBackendFlags_None :: None;
    ImGuiBackendFlags_HasGamepad :: HasGamepad;
    ImGuiBackendFlags_HasMouseCursors :: HasMouseCursors;
    ImGuiBackendFlags_HasSetMousePos :: HasSetMousePos;
    ImGuiBackendFlags_RendererHasVtxOffset :: RendererHasVtxOffset;
}

ButtonFlags :: enum_flags s32 {
    None :: 0x0;
    MouseButtonLeft :: 0x1;
    MouseButtonRight :: 0x2;
    MouseButtonMiddle :: 0x4;
    MouseButtonMask_ :: 0x7;
    MouseButtonDefault_ :: 0x1;
    ImGuiButtonFlags_None :: None;
    ImGuiButtonFlags_MouseButtonLeft :: MouseButtonLeft;
    ImGuiButtonFlags_MouseButtonRight :: MouseButtonRight;
    ImGuiButtonFlags_MouseButtonMiddle :: MouseButtonMiddle;
    ImGuiButtonFlags_MouseButtonMask_ :: MouseButtonMask_;
    ImGuiButtonFlags_MouseButtonDefault_ :: MouseButtonDefault_;
}

Col :: enum s32 {
    Text :: 0;
    TextDisabled :: 1;
    WindowBg :: 2;
    ChildBg :: 3;
    PopupBg :: 4;
    Border :: 5;
    BorderShadow :: 6;
    FrameBg :: 7;
    FrameBgHovered :: 8;
    FrameBgActive :: 9;
    TitleBg :: 10;
    TitleBgActive :: 11;
    TitleBgCollapsed :: 12;
    MenuBarBg :: 13;
    ScrollbarBg :: 14;
    ScrollbarGrab :: 15;
    ScrollbarGrabHovered :: 16;
    ScrollbarGrabActive :: 17;
    CheckMark :: 18;
    SliderGrab :: 19;
    SliderGrabActive :: 20;
    Button :: 21;
    ButtonHovered :: 22;
    ButtonActive :: 23;
    Header :: 24;
    HeaderHovered :: 25;
    HeaderActive :: 26;
    Separator :: 27;
    SeparatorHovered :: 28;
    SeparatorActive :: 29;
    ResizeGrip :: 30;
    ResizeGripHovered :: 31;
    ResizeGripActive :: 32;
    Tab :: 33;
    TabHovered :: 34;
    TabActive :: 35;
    TabUnfocused :: 36;
    TabUnfocusedActive :: 37;
    PlotLines :: 38;
    PlotLinesHovered :: 39;
    PlotHistogram :: 40;
    PlotHistogramHovered :: 41;
    TableHeaderBg :: 42;
    TableBorderStrong :: 43;
    TableBorderLight :: 44;
    TableRowBg :: 45;
    TableRowBgAlt :: 46;
    TextSelectedBg :: 47;
    DragDropTarget :: 48;
    NavHighlight :: 49;
    NavWindowingHighlight :: 50;
    NavWindowingDimBg :: 51;
    ModalWindowDimBg :: 52;
    COUNT :: 53;
    ImGuiCol_Text :: Text;
    ImGuiCol_TextDisabled :: TextDisabled;
    ImGuiCol_WindowBg :: WindowBg;
    ImGuiCol_ChildBg :: ChildBg;
    ImGuiCol_PopupBg :: PopupBg;
    ImGuiCol_Border :: Border;
    ImGuiCol_BorderShadow :: BorderShadow;
    ImGuiCol_FrameBg :: FrameBg;
    ImGuiCol_FrameBgHovered :: FrameBgHovered;
    ImGuiCol_FrameBgActive :: FrameBgActive;
    ImGuiCol_TitleBg :: TitleBg;
    ImGuiCol_TitleBgActive :: TitleBgActive;
    ImGuiCol_TitleBgCollapsed :: TitleBgCollapsed;
    ImGuiCol_MenuBarBg :: MenuBarBg;
    ImGuiCol_ScrollbarBg :: ScrollbarBg;
    ImGuiCol_ScrollbarGrab :: ScrollbarGrab;
    ImGuiCol_ScrollbarGrabHovered :: ScrollbarGrabHovered;
    ImGuiCol_ScrollbarGrabActive :: ScrollbarGrabActive;
    ImGuiCol_CheckMark :: CheckMark;
    ImGuiCol_SliderGrab :: SliderGrab;
    ImGuiCol_SliderGrabActive :: SliderGrabActive;
    ImGuiCol_Button :: Button;
    ImGuiCol_ButtonHovered :: ButtonHovered;
    ImGuiCol_ButtonActive :: ButtonActive;
    ImGuiCol_Header :: Header;
    ImGuiCol_HeaderHovered :: HeaderHovered;
    ImGuiCol_HeaderActive :: HeaderActive;
    ImGuiCol_Separator :: Separator;
    ImGuiCol_SeparatorHovered :: SeparatorHovered;
    ImGuiCol_SeparatorActive :: SeparatorActive;
    ImGuiCol_ResizeGrip :: ResizeGrip;
    ImGuiCol_ResizeGripHovered :: ResizeGripHovered;
    ImGuiCol_ResizeGripActive :: ResizeGripActive;
    ImGuiCol_Tab :: Tab;
    ImGuiCol_TabHovered :: TabHovered;
    ImGuiCol_TabActive :: TabActive;
    ImGuiCol_TabUnfocused :: TabUnfocused;
    ImGuiCol_TabUnfocusedActive :: TabUnfocusedActive;
    ImGuiCol_PlotLines :: PlotLines;
    ImGuiCol_PlotLinesHovered :: PlotLinesHovered;
    ImGuiCol_PlotHistogram :: PlotHistogram;
    ImGuiCol_PlotHistogramHovered :: PlotHistogramHovered;
    ImGuiCol_TableHeaderBg :: TableHeaderBg;
    ImGuiCol_TableBorderStrong :: TableBorderStrong;
    ImGuiCol_TableBorderLight :: TableBorderLight;
    ImGuiCol_TableRowBg :: TableRowBg;
    ImGuiCol_TableRowBgAlt :: TableRowBgAlt;
    ImGuiCol_TextSelectedBg :: TextSelectedBg;
    ImGuiCol_DragDropTarget :: DragDropTarget;
    ImGuiCol_NavHighlight :: NavHighlight;
    ImGuiCol_NavWindowingHighlight :: NavWindowingHighlight;
    ImGuiCol_NavWindowingDimBg :: NavWindowingDimBg;
    ImGuiCol_ModalWindowDimBg :: ModalWindowDimBg;
    ImGuiCol_COUNT :: COUNT;
}

ColorEditFlags :: enum_flags s32 {
    None :: 0x0;
    NoAlpha :: 0x2;
    NoPicker :: 0x4;
    NoOptions :: 0x8;
    NoSmallPreview :: 0x10;
    NoInputs :: 0x20;
    NoTooltip :: 0x40;
    NoLabel :: 0x80;
    NoSidePreview :: 0x100;
    NoDragDrop :: 0x200;
    NoBorder :: 0x400;
    AlphaBar :: 0x10000;
    AlphaPreview :: 0x20000;
    AlphaPreviewHalf :: 0x40000;
    HDR :: 0x80000;
    DisplayRGB :: 0x100000;
    DisplayHSV :: 0x200000;
    DisplayHex :: 0x400000;
    Uint8 :: 0x800000;
    Float :: 0x1000000;
    PickerHueBar :: 0x2000000;
    PickerHueWheel :: 0x4000000;
    InputRGB :: 0x8000000;
    InputHSV :: 0x10000000;
    DefaultOptions_ :: 0xa900000;
    DisplayMask_ :: 0x700000;
    DataTypeMask_ :: 0x1800000;
    PickerMask_ :: 0x6000000;
    InputMask_ :: 0x18000000;
    ImGuiColorEditFlags_None :: None;
    ImGuiColorEditFlags_NoAlpha :: NoAlpha;
    ImGuiColorEditFlags_NoPicker :: NoPicker;
    ImGuiColorEditFlags_NoOptions :: NoOptions;
    ImGuiColorEditFlags_NoSmallPreview :: NoSmallPreview;
    ImGuiColorEditFlags_NoInputs :: NoInputs;
    ImGuiColorEditFlags_NoTooltip :: NoTooltip;
    ImGuiColorEditFlags_NoLabel :: NoLabel;
    ImGuiColorEditFlags_NoSidePreview :: NoSidePreview;
    ImGuiColorEditFlags_NoDragDrop :: NoDragDrop;
    ImGuiColorEditFlags_NoBorder :: NoBorder;
    ImGuiColorEditFlags_AlphaBar :: AlphaBar;
    ImGuiColorEditFlags_AlphaPreview :: AlphaPreview;
    ImGuiColorEditFlags_AlphaPreviewHalf :: AlphaPreviewHalf;
    ImGuiColorEditFlags_HDR :: HDR;
    ImGuiColorEditFlags_DisplayRGB :: DisplayRGB;
    ImGuiColorEditFlags_DisplayHSV :: DisplayHSV;
    ImGuiColorEditFlags_DisplayHex :: DisplayHex;
    ImGuiColorEditFlags_Uint8 :: Uint8;
    ImGuiColorEditFlags_Float :: Float;
    ImGuiColorEditFlags_PickerHueBar :: PickerHueBar;
    ImGuiColorEditFlags_PickerHueWheel :: PickerHueWheel;
    ImGuiColorEditFlags_InputRGB :: InputRGB;
    ImGuiColorEditFlags_InputHSV :: InputHSV;
    ImGuiColorEditFlags_DefaultOptions_ :: DefaultOptions_;
    ImGuiColorEditFlags_DisplayMask_ :: DisplayMask_;
    ImGuiColorEditFlags_DataTypeMask_ :: DataTypeMask_;
    ImGuiColorEditFlags_PickerMask_ :: PickerMask_;
    ImGuiColorEditFlags_InputMask_ :: InputMask_;
}

ComboFlags :: enum_flags s32 {
    None :: 0x0;
    PopupAlignLeft :: 0x1;
    HeightSmall :: 0x2;
    HeightRegular :: 0x4;
    HeightLarge :: 0x8;
    HeightLargest :: 0x10;
    NoArrowButton :: 0x20;
    NoPreview :: 0x40;
    HeightMask_ :: 0x1e;
    ImGuiComboFlags_None :: None;
    ImGuiComboFlags_PopupAlignLeft :: PopupAlignLeft;
    ImGuiComboFlags_HeightSmall :: HeightSmall;
    ImGuiComboFlags_HeightRegular :: HeightRegular;
    ImGuiComboFlags_HeightLarge :: HeightLarge;
    ImGuiComboFlags_HeightLargest :: HeightLargest;
    ImGuiComboFlags_NoArrowButton :: NoArrowButton;
    ImGuiComboFlags_NoPreview :: NoPreview;
    ImGuiComboFlags_HeightMask_ :: HeightMask_;
}

Cond :: enum s32 {
    None :: 0;
    Always :: 1;
    Once :: 2;
    FirstUseEver :: 4;
    Appearing :: 8;
    ImGuiCond_None :: None;
    ImGuiCond_Always :: Always;
    ImGuiCond_Once :: Once;
    ImGuiCond_FirstUseEver :: FirstUseEver;
    ImGuiCond_Appearing :: Appearing;
}

ConfigFlags :: enum_flags s32 {
    None :: 0x0;
    NavEnableKeyboard :: 0x1;
    NavEnableGamepad :: 0x2;
    NavEnableSetMousePos :: 0x4;
    NavNoCaptureKeyboard :: 0x8;
    NoMouse :: 0x10;
    NoMouseCursorChange :: 0x20;
    IsSRGB :: 0x100000;
    IsTouchScreen :: 0x200000;
    ImGuiConfigFlags_None :: None;
    ImGuiConfigFlags_NavEnableKeyboard :: NavEnableKeyboard;
    ImGuiConfigFlags_NavEnableGamepad :: NavEnableGamepad;
    ImGuiConfigFlags_NavEnableSetMousePos :: NavEnableSetMousePos;
    ImGuiConfigFlags_NavNoCaptureKeyboard :: NavNoCaptureKeyboard;
    ImGuiConfigFlags_NoMouse :: NoMouse;
    ImGuiConfigFlags_NoMouseCursorChange :: NoMouseCursorChange;
    ImGuiConfigFlags_IsSRGB :: IsSRGB;
    ImGuiConfigFlags_IsTouchScreen :: IsTouchScreen;
}

DataType :: enum s32 {
    S8 :: 0;
    U8 :: 1;
    S16 :: 2;
    U16 :: 3;
    S32 :: 4;
    U32 :: 5;
    S64 :: 6;
    U64 :: 7;
    Float :: 8;
    Double :: 9;
    COUNT :: 10;
    ImGuiDataType_S8 :: S8;
    ImGuiDataType_U8 :: U8;
    ImGuiDataType_S16 :: S16;
    ImGuiDataType_U16 :: U16;
    ImGuiDataType_S32 :: S32;
    ImGuiDataType_U32 :: U32;
    ImGuiDataType_S64 :: S64;
    ImGuiDataType_U64 :: U64;
    ImGuiDataType_Float :: Float;
    ImGuiDataType_Double :: Double;
    ImGuiDataType_COUNT :: COUNT;
}

Dir :: enum s32 {
    None :: -1;
    Left :: 0;
    Right :: 1;
    Up :: 2;
    Down :: 3;
    COUNT :: 4;
    ImGuiDir_None :: None;
    ImGuiDir_Left :: Left;
    ImGuiDir_Right :: Right;
    ImGuiDir_Up :: Up;
    ImGuiDir_Down :: Down;
    ImGuiDir_COUNT :: COUNT;
}

DragDropFlags :: enum_flags s32 {
    None :: 0x0;
    SourceNoPreviewTooltip :: 0x1;
    SourceNoDisableHover :: 0x2;
    SourceNoHoldToOpenOthers :: 0x4;
    SourceAllowNullID :: 0x8;
    SourceExtern :: 0x10;
    SourceAutoExpirePayload :: 0x20;
    AcceptBeforeDelivery :: 0x400;
    AcceptNoDrawDefaultRect :: 0x800;
    AcceptNoPreviewTooltip :: 0x1000;
    AcceptPeekOnly :: 0xc00;
    ImGuiDragDropFlags_None :: None;
    ImGuiDragDropFlags_SourceNoPreviewTooltip :: SourceNoPreviewTooltip;
    ImGuiDragDropFlags_SourceNoDisableHover :: SourceNoDisableHover;
    ImGuiDragDropFlags_SourceNoHoldToOpenOthers :: SourceNoHoldToOpenOthers;
    ImGuiDragDropFlags_SourceAllowNullID :: SourceAllowNullID;
    ImGuiDragDropFlags_SourceExtern :: SourceExtern;
    ImGuiDragDropFlags_SourceAutoExpirePayload :: SourceAutoExpirePayload;
    ImGuiDragDropFlags_AcceptBeforeDelivery :: AcceptBeforeDelivery;
    ImGuiDragDropFlags_AcceptNoDrawDefaultRect :: AcceptNoDrawDefaultRect;
    ImGuiDragDropFlags_AcceptNoPreviewTooltip :: AcceptNoPreviewTooltip;
    ImGuiDragDropFlags_AcceptPeekOnly :: AcceptPeekOnly;
}

FocusedFlags :: enum_flags s32 {
    None :: 0x0;
    ChildWindows :: 0x1;
    RootWindow :: 0x2;
    AnyWindow :: 0x4;
    NoPopupHierarchy :: 0x8;
    RootAndChildWindows :: 0x3;
    ImGuiFocusedFlags_None :: None;
    ImGuiFocusedFlags_ChildWindows :: ChildWindows;
    ImGuiFocusedFlags_RootWindow :: RootWindow;
    ImGuiFocusedFlags_AnyWindow :: AnyWindow;
    ImGuiFocusedFlags_NoPopupHierarchy :: NoPopupHierarchy;
    ImGuiFocusedFlags_RootAndChildWindows :: RootAndChildWindows;
}

HoveredFlags :: enum_flags s32 {
    None :: 0x0;
    ChildWindows :: 0x1;
    RootWindow :: 0x2;
    AnyWindow :: 0x4;
    NoPopupHierarchy :: 0x8;
    AllowWhenBlockedByPopup :: 0x20;
    AllowWhenBlockedByActiveItem :: 0x80;
    AllowWhenOverlapped :: 0x100;
    AllowWhenDisabled :: 0x200;
    NoNavOverride :: 0x400;
    RectOnly :: 0x1a0;
    RootAndChildWindows :: 0x3;
    DelayNormal :: 0x800;
    DelayShort :: 0x1000;
    NoSharedDelay :: 0x2000;
    ImGuiHoveredFlags_None :: None;
    ImGuiHoveredFlags_ChildWindows :: ChildWindows;
    ImGuiHoveredFlags_RootWindow :: RootWindow;
    ImGuiHoveredFlags_AnyWindow :: AnyWindow;
    ImGuiHoveredFlags_NoPopupHierarchy :: NoPopupHierarchy;
    ImGuiHoveredFlags_AllowWhenBlockedByPopup :: AllowWhenBlockedByPopup;
    ImGuiHoveredFlags_AllowWhenBlockedByActiveItem :: AllowWhenBlockedByActiveItem;
    ImGuiHoveredFlags_AllowWhenOverlapped :: AllowWhenOverlapped;
    ImGuiHoveredFlags_AllowWhenDisabled :: AllowWhenDisabled;
    ImGuiHoveredFlags_NoNavOverride :: NoNavOverride;
    ImGuiHoveredFlags_RectOnly :: RectOnly;
    ImGuiHoveredFlags_RootAndChildWindows :: RootAndChildWindows;
    ImGuiHoveredFlags_DelayNormal :: DelayNormal;
    ImGuiHoveredFlags_DelayShort :: DelayShort;
    ImGuiHoveredFlags_NoSharedDelay :: NoSharedDelay;
}

ImDrawChannel :: struct {
    _CmdBuffer: ImVector(ImDrawCmd);
    _IdxBuffer: ImVector(ImDrawIdx);
}

ImDrawCmd :: struct {
    ClipRect: ImVec4;
    TextureId: ImTextureID;
    VtxOffset: u32;
    IdxOffset: u32;
    ElemCount: u32;
    UserCallback: ImDrawCallback;
    UserCallbackData: *void;
}

ImDrawCmdHeader :: struct {
    ClipRect: ImVec4;
    TextureId: ImTextureID;
    VtxOffset: u32;
}

ImDrawCornerFlags :: enum_flags s32 {
    None :: 0x100;
    TopLeft :: 0x10;
    TopRight :: 0x20;
    BotLeft :: 0x40;
    BotRight :: 0x80;
    All :: 0xf0;
    Top :: 0x30;
    Bot :: 0xc0;
    Left :: 0x50;
    Right :: 0xa0;
    ImDrawCornerFlags_None :: None;
    ImDrawCornerFlags_TopLeft :: TopLeft;
    ImDrawCornerFlags_TopRight :: TopRight;
    ImDrawCornerFlags_BotLeft :: BotLeft;
    ImDrawCornerFlags_BotRight :: BotRight;
    ImDrawCornerFlags_All :: All;
    ImDrawCornerFlags_Top :: Top;
    ImDrawCornerFlags_Bot :: Bot;
    ImDrawCornerFlags_Left :: Left;
    ImDrawCornerFlags_Right :: Right;
}

ImDrawData :: struct {
    Valid: bool;
    CmdListsCount: s32;
    TotalIdxCount: s32;
    TotalVtxCount: s32;
    CmdLists: **ImDrawList;
    DisplayPos: ImVec2;
    DisplaySize: ImVec2;
    FramebufferScale: ImVec2;
    DeIndexAllBuffers :: (this: *ImDrawData) -> void #cpp_method #foreign imgui "?DeIndexAllBuffers@ImDrawData@@QEAAXXZ";
    ScaleClipRects :: (this: *ImDrawData, fb_scale: *ImVec2) -> void #cpp_method #foreign imgui "?ScaleClipRects@ImDrawData@@QEAAXAEBUImVec2@@@Z";
    ScaleClipRects :: (this: *ImDrawData, fb_scale: ImVec2) #no_context {
        ScaleClipRects(this, *fb_scale);
    }
}

ImDrawFlags :: enum_flags s32 {
    None :: 0x0;
    Closed :: 0x1;
    RoundCornersTopLeft :: 0x10;
    RoundCornersTopRight :: 0x20;
    RoundCornersBottomLeft :: 0x40;
    RoundCornersBottomRight :: 0x80;
    RoundCornersNone :: 0x100;
    RoundCornersTop :: 0x30;
    RoundCornersBottom :: 0xc0;
    RoundCornersLeft :: 0x50;
    RoundCornersRight :: 0xa0;
    RoundCornersAll :: 0xf0;
    RoundCornersDefault_ :: 0xf0;
    RoundCornersMask_ :: 0x1f0;
    ImDrawFlags_None :: None;
    ImDrawFlags_Closed :: Closed;
    ImDrawFlags_RoundCornersTopLeft :: RoundCornersTopLeft;
    ImDrawFlags_RoundCornersTopRight :: RoundCornersTopRight;
    ImDrawFlags_RoundCornersBottomLeft :: RoundCornersBottomLeft;
    ImDrawFlags_RoundCornersBottomRight :: RoundCornersBottomRight;
    ImDrawFlags_RoundCornersNone :: RoundCornersNone;
    ImDrawFlags_RoundCornersTop :: RoundCornersTop;
    ImDrawFlags_RoundCornersBottom :: RoundCornersBottom;
    ImDrawFlags_RoundCornersLeft :: RoundCornersLeft;
    ImDrawFlags_RoundCornersRight :: RoundCornersRight;
    ImDrawFlags_RoundCornersAll :: RoundCornersAll;
    ImDrawFlags_RoundCornersDefault_ :: RoundCornersDefault_;
    ImDrawFlags_RoundCornersMask_ :: RoundCornersMask_;
}

ImDrawList :: struct {
    CmdBuffer: ImVector(ImDrawCmd);
    IdxBuffer: ImVector(ImDrawIdx);
    VtxBuffer: ImVector(ImDrawVert);
    Flags: ImDrawListFlags;
    _VtxCurrentIdx: u32;
    _Data: *ImDrawListSharedData;
    _OwnerName: *u8;
    _VtxWritePtr: *ImDrawVert;
    _IdxWritePtr: *ImDrawIdx;
    _ClipRectStack: ImVector(ImVec4);
    _TextureIdStack: ImVector(ImTextureID);
    _Path: ImVector(ImVec2);
    _CmdHeader: ImDrawCmdHeader;
    _Splitter: ImDrawListSplitter;
    _FringeScale: float;
    PushClipRect :: (this: *ImDrawList, clip_rect_min: *ImVec2, clip_rect_max: *ImVec2, intersect_with_current_clip_rect := false) -> void #cpp_method #foreign imgui "?PushClipRect@ImDrawList@@QEAAXAEBUImVec2@@0_N@Z";
    PushClipRect :: (this: *ImDrawList, clip_rect_min: ImVec2, clip_rect_max: ImVec2, intersect_with_current_clip_rect := false) #no_context {
        PushClipRect(this, *clip_rect_min, *clip_rect_max, intersect_with_current_clip_rect);
    }
    PushClipRectFullScreen :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?PushClipRectFullScreen@ImDrawList@@QEAAXXZ";
    PopClipRect :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?PopClipRect@ImDrawList@@QEAAXXZ";
    PushTextureID :: (this: *ImDrawList, texture_id: ImTextureID) -> void #cpp_method #foreign imgui "?PushTextureID@ImDrawList@@QEAAXPEAX@Z";
    PopTextureID :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?PopTextureID@ImDrawList@@QEAAXXZ";
    AddLine :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, col: ImU32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "?AddLine@ImDrawList@@QEAAXAEBUImVec2@@0IM@Z";
    AddLine :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, col: ImU32, thickness: float = 1.0) #no_context {
        AddLine(this, *p1, *p2, col, thickness);
    }
    AddRect :: (this: *ImDrawList, p_min: *ImVec2, p_max: *ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None, thickness: float = 1.0) -> void #cpp_method #foreign imgui "?AddRect@ImDrawList@@QEAAXAEBUImVec2@@0IMHM@Z";
    AddRect :: (this: *ImDrawList, p_min: ImVec2, p_max: ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None, thickness: float = 1.0) #no_context {
        AddRect(this, *p_min, *p_max, col, rounding, flags, thickness);
    }
    AddRectFilled :: (this: *ImDrawList, p_min: *ImVec2, p_max: *ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None) -> void #cpp_method #foreign imgui "?AddRectFilled@ImDrawList@@QEAAXAEBUImVec2@@0IMH@Z";
    AddRectFilled :: (this: *ImDrawList, p_min: ImVec2, p_max: ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None) #no_context {
        AddRectFilled(this, *p_min, *p_max, col, rounding, flags);
    }
    AddRectFilledMultiColor :: (this: *ImDrawList, p_min: *ImVec2, p_max: *ImVec2, col_upr_left: ImU32, col_upr_right: ImU32, col_bot_right: ImU32, col_bot_left: ImU32) -> void #cpp_method #foreign imgui "?AddRectFilledMultiColor@ImDrawList@@QEAAXAEBUImVec2@@0IIII@Z";
    AddRectFilledMultiColor :: (this: *ImDrawList, p_min: ImVec2, p_max: ImVec2, col_upr_left: ImU32, col_upr_right: ImU32, col_bot_right: ImU32, col_bot_left: ImU32) #no_context {
        AddRectFilledMultiColor(this, *p_min, *p_max, col_upr_left, col_upr_right, col_bot_right, col_bot_left);
    }
    AddQuad :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, col: ImU32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "?AddQuad@ImDrawList@@QEAAXAEBUImVec2@@000IM@Z";
    AddQuad :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, col: ImU32, thickness: float = 1.0) #no_context {
        AddQuad(this, *p1, *p2, *p3, *p4, col, thickness);
    }
    AddQuadFilled :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "?AddQuadFilled@ImDrawList@@QEAAXAEBUImVec2@@000I@Z";
    AddQuadFilled :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, col: ImU32) #no_context {
        AddQuadFilled(this, *p1, *p2, *p3, *p4, col);
    }
    AddTriangle :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, col: ImU32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "?AddTriangle@ImDrawList@@QEAAXAEBUImVec2@@00IM@Z";
    AddTriangle :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, col: ImU32, thickness: float = 1.0) #no_context {
        AddTriangle(this, *p1, *p2, *p3, col, thickness);
    }
    AddTriangleFilled :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "?AddTriangleFilled@ImDrawList@@QEAAXAEBUImVec2@@00I@Z";
    AddTriangleFilled :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, col: ImU32) #no_context {
        AddTriangleFilled(this, *p1, *p2, *p3, col);
    }
    AddCircle :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32 = 0, thickness: float = 1.0) -> void #cpp_method #foreign imgui "?AddCircle@ImDrawList@@QEAAXAEBUImVec2@@MIHM@Z";
    AddCircle :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32 = 0, thickness: float = 1.0) #no_context {
        AddCircle(this, *center, radius, col, num_segments, thickness);
    }
    AddCircleFilled :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "?AddCircleFilled@ImDrawList@@QEAAXAEBUImVec2@@MIH@Z";
    AddCircleFilled :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32 = 0) #no_context {
        AddCircleFilled(this, *center, radius, col, num_segments);
    }
    AddNgon :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "?AddNgon@ImDrawList@@QEAAXAEBUImVec2@@MIHM@Z";
    AddNgon :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32, thickness: float = 1.0) #no_context {
        AddNgon(this, *center, radius, col, num_segments, thickness);
    }
    AddNgonFilled :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32) -> void #cpp_method #foreign imgui "?AddNgonFilled@ImDrawList@@QEAAXAEBUImVec2@@MIH@Z";
    AddNgonFilled :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32) #no_context {
        AddNgonFilled(this, *center, radius, col, num_segments);
    }
    AddText :: (this: *ImDrawList, pos: *ImVec2, col: ImU32, text_begin: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "?AddText@ImDrawList@@QEAAXAEBUImVec2@@IPEBD1@Z";
    AddText :: (this: *ImDrawList, pos: ImVec2, col: ImU32, text: string) #no_context {
        AddText(this, *pos, col, text.data, text.data + text.count);
    }
    AddText :: (this: *ImDrawList, font: *ImFont, font_size: float, pos: *ImVec2, col: ImU32, text_begin: *u8, text_end: *u8 = null, wrap_width: float = 0.0, cpu_fine_clip_rect: *ImVec4 = null) -> void #cpp_method #foreign imgui "?AddText@ImDrawList@@QEAAXPEBUImFont@@MAEBUImVec2@@IPEBD2MPEBUImVec4@@@Z";
    AddText :: (this: *ImDrawList, font: *ImFont, font_size: float, pos: ImVec2, col: ImU32, text: string, wrap_width: float = 0.0, cpu_fine_clip_rect: *ImVec4 = null) #no_context {
        AddText(this, font, font_size, *pos, col, text.data, text.data + text.count, wrap_width, cpu_fine_clip_rect);
    }
    AddPolyline :: (this: *ImDrawList, points: *ImVec2, num_points: s32, col: ImU32, flags: ImDrawFlags, thickness: float) -> void #cpp_method #foreign imgui "?AddPolyline@ImDrawList@@QEAAXPEBUImVec2@@HIHM@Z";
    AddConvexPolyFilled :: (this: *ImDrawList, points: *ImVec2, num_points: s32, col: ImU32) -> void #cpp_method #foreign imgui "?AddConvexPolyFilled@ImDrawList@@QEAAXPEBUImVec2@@HI@Z";
    AddBezierCubic :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "?AddBezierCubic@ImDrawList@@QEAAXAEBUImVec2@@000IMH@Z";
    AddBezierCubic :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) #no_context {
        AddBezierCubic(this, *p1, *p2, *p3, *p4, col, thickness, num_segments);
    }
    AddBezierQuadratic :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "?AddBezierQuadratic@ImDrawList@@QEAAXAEBUImVec2@@00IMH@Z";
    AddBezierQuadratic :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) #no_context {
        AddBezierQuadratic(this, *p1, *p2, *p3, col, thickness, num_segments);
    }
    AddImage :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: *ImVec2, p_max: *ImVec2, uv_min: *ImVec2, uv_max: *ImVec2, col: ImU32 = 4294967295) -> void #cpp_method #foreign imgui "?AddImage@ImDrawList@@QEAAXPEAXAEBUImVec2@@111I@Z";
    AddImage :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: ImVec2, p_max: ImVec2, uv_min: ImVec2 = ImVec2.{ 0, 0 }, uv_max: ImVec2 = ImVec2.{ 1, 1 }, col: ImU32 = 4294967295) #no_context {
        AddImage(this, user_texture_id, *p_min, *p_max, *uv_min, *uv_max, col);
    }
    AddImageQuad :: (this: *ImDrawList, user_texture_id: ImTextureID, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, uv1: *ImVec2, uv2: *ImVec2, uv3: *ImVec2, uv4: *ImVec2, col: ImU32 = 4294967295) -> void #cpp_method #foreign imgui "?AddImageQuad@ImDrawList@@QEAAXPEAXAEBUImVec2@@1111111I@Z";
    AddImageQuad :: (this: *ImDrawList, user_texture_id: ImTextureID, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, uv1: ImVec2 = ImVec2.{ 0, 0 }, uv2: ImVec2 = ImVec2.{ 1, 0 }, uv3: ImVec2 = ImVec2.{ 1, 1 }, uv4: ImVec2 = ImVec2.{ 0, 1 }, col: ImU32 = 4294967295) #no_context {
        AddImageQuad(this, user_texture_id, *p1, *p2, *p3, *p4, *uv1, *uv2, *uv3, *uv4, col);
    }
    AddImageRounded :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: *ImVec2, p_max: *ImVec2, uv_min: *ImVec2, uv_max: *ImVec2, col: ImU32, rounding: float, flags: ImDrawFlags = .None) -> void #cpp_method #foreign imgui "?AddImageRounded@ImDrawList@@QEAAXPEAXAEBUImVec2@@111IMH@Z";
    AddImageRounded :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: ImVec2, p_max: ImVec2, uv_min: ImVec2, uv_max: ImVec2, col: ImU32, rounding: float, flags: ImDrawFlags = .None) #no_context {
        AddImageRounded(this, user_texture_id, *p_min, *p_max, *uv_min, *uv_max, col, rounding, flags);
    }
    PathArcTo :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "?PathArcTo@ImDrawList@@QEAAXAEBUImVec2@@MMMH@Z";
    PathArcTo :: (this: *ImDrawList, center: ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32 = 0) #no_context {
        PathArcTo(this, *center, radius, a_min, a_max, num_segments);
    }
    PathArcToFast :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min_of_12: s32, a_max_of_12: s32) -> void #cpp_method #foreign imgui "?PathArcToFast@ImDrawList@@QEAAXAEBUImVec2@@MHH@Z";
    PathArcToFast :: (this: *ImDrawList, center: ImVec2, radius: float, a_min_of_12: s32, a_max_of_12: s32) #no_context {
        PathArcToFast(this, *center, radius, a_min_of_12, a_max_of_12);
    }
    PathBezierCubicCurveTo :: (this: *ImDrawList, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "?PathBezierCubicCurveTo@ImDrawList@@QEAAXAEBUImVec2@@00H@Z";
    PathBezierCubicCurveTo :: (this: *ImDrawList, p2: ImVec2, p3: ImVec2, p4: ImVec2, num_segments: s32 = 0) #no_context {
        PathBezierCubicCurveTo(this, *p2, *p3, *p4, num_segments);
    }
    PathBezierQuadraticCurveTo :: (this: *ImDrawList, p2: *ImVec2, p3: *ImVec2, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "?PathBezierQuadraticCurveTo@ImDrawList@@QEAAXAEBUImVec2@@0H@Z";
    PathBezierQuadraticCurveTo :: (this: *ImDrawList, p2: ImVec2, p3: ImVec2, num_segments: s32 = 0) #no_context {
        PathBezierQuadraticCurveTo(this, *p2, *p3, num_segments);
    }
    PathRect :: (this: *ImDrawList, rect_min: *ImVec2, rect_max: *ImVec2, rounding: float = 0.0, flags: ImDrawFlags = .None) -> void #cpp_method #foreign imgui "?PathRect@ImDrawList@@QEAAXAEBUImVec2@@0MH@Z";
    PathRect :: (this: *ImDrawList, rect_min: ImVec2, rect_max: ImVec2, rounding: float = 0.0, flags: ImDrawFlags = .None) #no_context {
        PathRect(this, *rect_min, *rect_max, rounding, flags);
    }
    AddCallback :: (this: *ImDrawList, callback: ImDrawCallback, callback_data: *void) -> void #cpp_method #foreign imgui "?AddCallback@ImDrawList@@QEAAXP6AXPEBU1@PEBUImDrawCmd@@@ZPEAX@Z";
    AddDrawCmd :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?AddDrawCmd@ImDrawList@@QEAAXXZ";
    CloneOutput :: (this: *ImDrawList) -> *ImDrawList #cpp_method #foreign imgui "?CloneOutput@ImDrawList@@QEBAPEAU1@XZ";
    PrimReserve :: (this: *ImDrawList, idx_count: s32, vtx_count: s32) -> void #cpp_method #foreign imgui "?PrimReserve@ImDrawList@@QEAAXHH@Z";
    PrimUnreserve :: (this: *ImDrawList, idx_count: s32, vtx_count: s32) -> void #cpp_method #foreign imgui "?PrimUnreserve@ImDrawList@@QEAAXHH@Z";
    PrimRect :: (this: *ImDrawList, a: *ImVec2, b: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "?PrimRect@ImDrawList@@QEAAXAEBUImVec2@@0I@Z";
    PrimRect :: (this: *ImDrawList, a: ImVec2, b: ImVec2, col: ImU32) #no_context {
        PrimRect(this, *a, *b, col);
    }
    PrimRectUV :: (this: *ImDrawList, a: *ImVec2, b: *ImVec2, uv_a: *ImVec2, uv_b: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "?PrimRectUV@ImDrawList@@QEAAXAEBUImVec2@@000I@Z";
    PrimRectUV :: (this: *ImDrawList, a: ImVec2, b: ImVec2, uv_a: ImVec2, uv_b: ImVec2, col: ImU32) #no_context {
        PrimRectUV(this, *a, *b, *uv_a, *uv_b, col);
    }
    PrimQuadUV :: (this: *ImDrawList, a: *ImVec2, b: *ImVec2, c: *ImVec2, d: *ImVec2, uv_a: *ImVec2, uv_b: *ImVec2, uv_c: *ImVec2, uv_d: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "?PrimQuadUV@ImDrawList@@QEAAXAEBUImVec2@@0000000I@Z";
    PrimQuadUV :: (this: *ImDrawList, a: ImVec2, b: ImVec2, c: ImVec2, d: ImVec2, uv_a: ImVec2, uv_b: ImVec2, uv_c: ImVec2, uv_d: ImVec2, col: ImU32) #no_context {
        PrimQuadUV(this, *a, *b, *c, *d, *uv_a, *uv_b, *uv_c, *uv_d, col);
    }
    _ResetForNewFrame :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_ResetForNewFrame@ImDrawList@@QEAAXXZ";
    _ClearFreeMemory :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_ClearFreeMemory@ImDrawList@@QEAAXXZ";
    _PopUnusedDrawCmd :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_PopUnusedDrawCmd@ImDrawList@@QEAAXXZ";
    _TryMergeDrawCmds :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_TryMergeDrawCmds@ImDrawList@@QEAAXXZ";
    _OnChangedClipRect :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_OnChangedClipRect@ImDrawList@@QEAAXXZ";
    _OnChangedTextureID :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_OnChangedTextureID@ImDrawList@@QEAAXXZ";
    _OnChangedVtxOffset :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "?_OnChangedVtxOffset@ImDrawList@@QEAAXXZ";
    _CalcCircleAutoSegmentCount :: (this: *ImDrawList, radius: float) -> s32 #cpp_method #foreign imgui "?_CalcCircleAutoSegmentCount@ImDrawList@@QEBAHM@Z";
    _PathArcToFastEx :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min_sample: s32, a_max_sample: s32, a_step: s32) -> void #cpp_method #foreign imgui "?_PathArcToFastEx@ImDrawList@@QEAAXAEBUImVec2@@MHHH@Z";
    _PathArcToFastEx :: (this: *ImDrawList, center: ImVec2, radius: float, a_min_sample: s32, a_max_sample: s32, a_step: s32) #no_context {
        _PathArcToFastEx(this, *center, radius, a_min_sample, a_max_sample, a_step);
    }
    _PathArcToN :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32) -> void #cpp_method #foreign imgui "?_PathArcToN@ImDrawList@@QEAAXAEBUImVec2@@MMMH@Z";
    _PathArcToN :: (this: *ImDrawList, center: ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32) #no_context {
        _PathArcToN(this, *center, radius, a_min, a_max, num_segments);
    }
}

ImDrawListFlags :: enum_flags s32 {
    None :: 0x0;
    AntiAliasedLines :: 0x1;
    AntiAliasedLinesUseTex :: 0x2;
    AntiAliasedFill :: 0x4;
    AllowVtxOffset :: 0x8;
    ImDrawListFlags_None :: None;
    ImDrawListFlags_AntiAliasedLines :: AntiAliasedLines;
    ImDrawListFlags_AntiAliasedLinesUseTex :: AntiAliasedLinesUseTex;
    ImDrawListFlags_AntiAliasedFill :: AntiAliasedFill;
    ImDrawListFlags_AllowVtxOffset :: AllowVtxOffset;
}

ImDrawListSharedData :: struct {}

ImDrawListSplitter :: struct {
    _Current: s32;
    _Count: s32;
    _Channels: ImVector(ImDrawChannel);
    ClearFreeMemory :: (this: *ImDrawListSplitter) -> void #cpp_method #foreign imgui "?ClearFreeMemory@ImDrawListSplitter@@QEAAXXZ";
    Split :: (this: *ImDrawListSplitter, draw_list: *ImDrawList, count: s32) -> void #cpp_method #foreign imgui "?Split@ImDrawListSplitter@@QEAAXPEAUImDrawList@@H@Z";
    Merge :: (this: *ImDrawListSplitter, draw_list: *ImDrawList) -> void #cpp_method #foreign imgui "?Merge@ImDrawListSplitter@@QEAAXPEAUImDrawList@@@Z";
    SetCurrentChannel :: (this: *ImDrawListSplitter, draw_list: *ImDrawList, channel_idx: s32) -> void #cpp_method #foreign imgui "?SetCurrentChannel@ImDrawListSplitter@@QEAAXPEAUImDrawList@@H@Z";
}

ImDrawVert :: struct {
    pos: ImVec2;
    uv: ImVec2;
    col: ImU32;
}

ImFont :: struct {
    IndexAdvanceX: ImVector(float);
    FallbackAdvanceX: float;
    FontSize: float;
    IndexLookup: ImVector(ImWchar);
    Glyphs: ImVector(ImFontGlyph);
    FallbackGlyph: *ImFontGlyph;
    ContainerAtlas: *ImFontAtlas;
    ConfigData: *ImFontConfig;
    ConfigDataCount: s16;
    FallbackChar: ImWchar;
    EllipsisChar: ImWchar;
    EllipsisCharCount: s16;
    EllipsisWidth: float;
    EllipsisCharStep: float;
    DirtyLookupTables: bool;
    Scale: float;
    Ascent: float;
    Descent: float;
    MetricsTotalSurface: s32;
    Used4kPagesMap: [2] ImU8;
    Constructor :: (this: *ImFont) -> void #cpp_method #foreign imgui "??0ImFont@@QEAA@XZ";
    Destructor :: (this: *ImFont) -> void #cpp_method #foreign imgui "??1ImFont@@QEAA@XZ";
    FindGlyph :: (this: *ImFont, c: ImWchar) -> *ImFontGlyph #cpp_method #foreign imgui "?FindGlyph@ImFont@@QEBAPEBUImFontGlyph@@G@Z";
    FindGlyphNoFallback :: (this: *ImFont, c: ImWchar) -> *ImFontGlyph #cpp_method #foreign imgui "?FindGlyphNoFallback@ImFont@@QEBAPEBUImFontGlyph@@G@Z";
    CalcTextSizeA :: (this: *ImFont, size: float, max_width: float, wrap_width: float, text_begin: *u8, text_end: *u8 = null, remaining: **u8 = null) -> ImVec2 #cpp_method #cpp_return_type_is_non_pod #foreign imgui "?CalcTextSizeA@ImFont@@QEBA?AUImVec2@@MMMPEBD0PEAPEBD@Z";
    CalcTextSizeA :: (this: *ImFont, size: float, max_width: float, wrap_width: float, text: string, remaining: **u8 = null) -> ImVec2 #no_context {
        return CalcTextSizeA(this, size, max_width, wrap_width, text.data, text.data + text.count, remaining);
    }
    CalcWordWrapPositionA :: (this: *ImFont, scale: float, text: *u8, text_end: *u8, wrap_width: float) -> *u8 #cpp_method #foreign imgui "?CalcWordWrapPositionA@ImFont@@QEBAPEBDMPEBD0M@Z";
    CalcWordWrapPositionA :: (this: *ImFont, scale: float, text: string, wrap_width: float) -> *u8 #no_context {
        return CalcWordWrapPositionA(this, scale, text.data, text.data + text.count, wrap_width);
    }
    RenderChar :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: *ImVec2, col: ImU32, c: ImWchar) -> void #cpp_method #foreign imgui "?RenderChar@ImFont@@QEBAXPEAUImDrawList@@MAEBUImVec2@@IG@Z";
    RenderChar :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: ImVec2, col: ImU32, c: ImWchar) #no_context {
        RenderChar(this, draw_list, size, *pos, col, c);
    }
    RenderText :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: *ImVec2, col: ImU32, clip_rect: *ImVec4, text_begin: *u8, text_end: *u8, wrap_width: float = 0.0, cpu_fine_clip := false) -> void #cpp_method #foreign imgui "?RenderText@ImFont@@QEBAXPEAUImDrawList@@MAEBUImVec2@@IAEBUImVec4@@PEBD3M_N@Z";
    RenderText :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: ImVec2, col: ImU32, clip_rect: ImVec4, text: string, wrap_width: float = 0.0, cpu_fine_clip := false) #no_context {
        RenderText(this, draw_list, size, *pos, col, *clip_rect, text.data, text.data + text.count, wrap_width, cpu_fine_clip);
    }
    BuildLookupTable :: (this: *ImFont) -> void #cpp_method #foreign imgui "?BuildLookupTable@ImFont@@QEAAXXZ";
    ClearOutputData :: (this: *ImFont) -> void #cpp_method #foreign imgui "?ClearOutputData@ImFont@@QEAAXXZ";
    GrowIndex :: (this: *ImFont, new_size: s32) -> void #cpp_method #foreign imgui "?GrowIndex@ImFont@@QEAAXH@Z";
    AddGlyph :: (this: *ImFont, src_cfg: *ImFontConfig, c: ImWchar, x0: float, y0: float, x1: float, y1: float, u0: float, v0: float, u1: float, v1: float, advance_x: float) -> void #cpp_method #foreign imgui "?AddGlyph@ImFont@@QEAAXPEBUImFontConfig@@GMMMMMMMMM@Z";
    AddRemapChar :: (this: *ImFont, dst: ImWchar, src: ImWchar, overwrite_dst := true) -> void #cpp_method #foreign imgui "?AddRemapChar@ImFont@@QEAAXGG_N@Z";
    SetGlyphVisible :: (this: *ImFont, c: ImWchar, visible: bool) -> void #cpp_method #foreign imgui "?SetGlyphVisible@ImFont@@QEAAXG_N@Z";
    IsGlyphRangeUnused :: (this: *ImFont, c_begin: u32, c_last: u32) -> bool #cpp_method #foreign imgui "?IsGlyphRangeUnused@ImFont@@QEAA_NII@Z";
}

ImFontAtlas :: struct {
    Constructor :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "??0ImFontAtlas@@QEAA@XZ";
    Destructor :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "??1ImFontAtlas@@QEAA@XZ";
    AddFont :: (this: *ImFontAtlas, font_cfg: *ImFontConfig) -> *ImFont #cpp_method #foreign imgui "?AddFont@ImFontAtlas@@QEAAPEAUImFont@@PEBUImFontConfig@@@Z";
    AddFontDefault :: (this: *ImFontAtlas, font_cfg: *ImFontConfig = null) -> *ImFont #cpp_method #foreign imgui "?AddFontDefault@ImFontAtlas@@QEAAPEAUImFont@@PEBUImFontConfig@@@Z";
    AddFontFromFileTTF :: (this: *ImFontAtlas, filename: *u8, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "?AddFontFromFileTTF@ImFontAtlas@@QEAAPEAUImFont@@PEBDMPEBUImFontConfig@@PEBG@Z";
    AddFontFromMemoryTTF :: (this: *ImFontAtlas, font_data: *void, font_size: s32, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "?AddFontFromMemoryTTF@ImFontAtlas@@QEAAPEAUImFont@@PEAXHMPEBUImFontConfig@@PEBG@Z";
    AddFontFromMemoryCompressedTTF :: (this: *ImFontAtlas, compressed_font_data: *void, compressed_font_size: s32, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "?AddFontFromMemoryCompressedTTF@ImFontAtlas@@QEAAPEAUImFont@@PEBXHMPEBUImFontConfig@@PEBG@Z";
    AddFontFromMemoryCompressedBase85TTF :: (this: *ImFontAtlas, compressed_font_data_base85: *u8, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "?AddFontFromMemoryCompressedBase85TTF@ImFontAtlas@@QEAAPEAUImFont@@PEBDMPEBUImFontConfig@@PEBG@Z";
    ClearInputData :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "?ClearInputData@ImFontAtlas@@QEAAXXZ";
    ClearTexData :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "?ClearTexData@ImFontAtlas@@QEAAXXZ";
    ClearFonts :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "?ClearFonts@ImFontAtlas@@QEAAXXZ";
    Clear :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "?Clear@ImFontAtlas@@QEAAXXZ";
    Build :: (this: *ImFontAtlas) -> bool #cpp_method #foreign imgui "?Build@ImFontAtlas@@QEAA_NXZ";
    GetTexDataAsAlpha8 :: (this: *ImFontAtlas, out_pixels: **u8, out_width: *s32, out_height: *s32, out_bytes_per_pixel: *s32 = null) -> void #cpp_method #foreign imgui "?GetTexDataAsAlpha8@ImFontAtlas@@QEAAXPEAPEAEPEAH11@Z";
    GetTexDataAsRGBA32 :: (this: *ImFontAtlas, out_pixels: **u8, out_width: *s32, out_height: *s32, out_bytes_per_pixel: *s32 = null) -> void #cpp_method #foreign imgui "?GetTexDataAsRGBA32@ImFontAtlas@@QEAAXPEAPEAEPEAH11@Z";
    GetGlyphRangesDefault :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesDefault@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesGreek :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesGreek@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesKorean :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesKorean@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesJapanese :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesJapanese@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesChineseFull :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesChineseFull@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesChineseSimplifiedCommon :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesChineseSimplifiedCommon@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesCyrillic :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesCyrillic@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesThai :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesThai@ImFontAtlas@@QEAAPEBGXZ";
    GetGlyphRangesVietnamese :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "?GetGlyphRangesVietnamese@ImFontAtlas@@QEAAPEBGXZ";
    AddCustomRectRegular :: (this: *ImFontAtlas, width: s32, height: s32) -> s32 #cpp_method #foreign imgui "?AddCustomRectRegular@ImFontAtlas@@QEAAHHH@Z";
    AddCustomRectFontGlyph :: (this: *ImFontAtlas, font: *ImFont, id: ImWchar, width: s32, height: s32, advance_x: float, offset: *ImVec2) -> s32 #cpp_method #foreign imgui "?AddCustomRectFontGlyph@ImFontAtlas@@QEAAHPEAUImFont@@GHHMAEBUImVec2@@@Z";
    AddCustomRectFontGlyph :: (this: *ImFontAtlas, font: *ImFont, id: ImWchar, width: s32, height: s32, advance_x: float, offset: ImVec2 = ImVec2.{ 0, 0 }) -> s32 #no_context {
        return AddCustomRectFontGlyph(this, font, id, width, height, advance_x, *offset);
    }
    CalcCustomRectUV :: (this: *ImFontAtlas, rect: *ImFontAtlasCustomRect, out_uv_min: *ImVec2, out_uv_max: *ImVec2) -> void #cpp_method #foreign imgui "?CalcCustomRectUV@ImFontAtlas@@QEBAXPEBUImFontAtlasCustomRect@@PEAUImVec2@@1@Z";
    GetMouseCursorTexData :: (this: *ImFontAtlas, cursor: MouseCursor, out_offset: *ImVec2, out_size: *ImVec2, out_uv_border: *[2] ImVec2, out_uv_fill: *[2] ImVec2) -> bool #cpp_method #foreign imgui "?GetMouseCursorTexData@ImFontAtlas@@QEAA_NHPEAUImVec2@@0QEAU2@1@Z";
    Flags: ImFontAtlasFlags;
    TexID: ImTextureID;
    TexDesiredWidth: s32;
    TexGlyphPadding: s32;
    Locked: bool;
    UserData: *void;
    TexReady: bool;
    TexPixelsUseColors: bool;
    TexPixelsAlpha8: *u8;
    TexPixelsRGBA32: *u32;
    TexWidth: s32;
    TexHeight: s32;
    TexUvScale: ImVec2;
    TexUvWhitePixel: ImVec2;
    Fonts: ImVector(*ImFont);
    CustomRects: ImVector(ImFontAtlasCustomRect);
    ConfigData: ImVector(ImFontConfig);
    TexUvLines: [64] ImVec4;
    FontBuilderIO: *ImFontBuilderIO;
    FontBuilderFlags: u32;
    PackIdMouseCursors: s32;
    PackIdLines: s32;
}

ImFontAtlasCustomRect :: struct {
    Width: u16;
    Height: u16;
    X: u16;
    Y: u16;
    GlyphID: u32;
    GlyphAdvanceX: float;
    GlyphOffset: ImVec2;
    Font: *ImFont;
}

ImFontAtlasFlags :: enum_flags s32 {
    None :: 0x0;
    NoPowerOfTwoHeight :: 0x1;
    NoMouseCursors :: 0x2;
    NoBakedLines :: 0x4;
    ImFontAtlasFlags_None :: None;
    ImFontAtlasFlags_NoPowerOfTwoHeight :: NoPowerOfTwoHeight;
    ImFontAtlasFlags_NoMouseCursors :: NoMouseCursors;
    ImFontAtlasFlags_NoBakedLines :: NoBakedLines;
}

ImFontBuilderIO :: struct {}

ImFontConfig :: struct {
    FontData: *void;
    FontDataSize: s32;
    FontDataOwnedByAtlas: bool;
    FontNo: s32;
    SizePixels: float;
    OversampleH: s32;
    OversampleV: s32;
    PixelSnapH: bool;
    GlyphExtraSpacing: ImVec2;
    GlyphOffset: ImVec2;
    GlyphRanges: *ImWchar;
    GlyphMinAdvanceX: float;
    GlyphMaxAdvanceX: float;
    MergeMode: bool;
    FontBuilderFlags: u32;
    RasterizerMultiply: float;
    EllipsisChar: ImWchar;
    Name: [40] u8;
    DstFont: *ImFont;
    Constructor :: (this: *ImFontConfig) -> void #cpp_method #foreign imgui "??0ImFontConfig@@QEAA@XZ";
}

ImFontGlyph :: struct {
    __bitfield: u32;
    AdvanceX: float;
    X0: float;
    Y0: float;
    X1: float;
    Y1: float;
    U0: float;
    V0: float;
    U1: float;
    V1: float;
}

ImFontGlyphRangesBuilder :: struct {
    UsedChars: ImVector(ImU32);
    AddText :: (this: *ImFontGlyphRangesBuilder, text: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "?AddText@ImFontGlyphRangesBuilder@@QEAAXPEBD0@Z";
    AddText :: (this: *ImFontGlyphRangesBuilder, text: string) #no_context {
        AddText(this, text.data, text.data + text.count);
    }
    AddRanges :: (this: *ImFontGlyphRangesBuilder, ranges: *ImWchar) -> void #cpp_method #foreign imgui "?AddRanges@ImFontGlyphRangesBuilder@@QEAAXPEBG@Z";
    BuildRanges :: (this: *ImFontGlyphRangesBuilder, out_ranges: *ImVector(ImWchar)) -> void #cpp_method #foreign imgui "?BuildRanges@ImFontGlyphRangesBuilder@@QEAAXPEAU?$ImVector@G@@@Z";
}

ImGuiContext :: struct {}

ImNewWrapper :: struct {
    __empty_struct_padding: u8;
}

InputTextCallbackData :: struct {
    Ctx: *ImGuiContext;
    EventFlag: InputTextFlags;
    Flags: InputTextFlags;
    UserData: *void;
    EventChar: ImWchar;
    EventKey: Key;
    Buf: *u8;
    BufTextLen: s32;
    BufSize: s32;
    BufDirty: bool;
    CursorPos: s32;
    SelectionStart: s32;
    SelectionEnd: s32;
    Constructor :: (this: *InputTextCallbackData) -> void #cpp_method #foreign imgui "??0ImGuiInputTextCallbackData@@QEAA@XZ";
    DeleteChars :: (this: *InputTextCallbackData, pos: s32, bytes_count: s32) -> void #cpp_method #foreign imgui "?DeleteChars@ImGuiInputTextCallbackData@@QEAAXHH@Z";
    InsertChars :: (this: *InputTextCallbackData, pos: s32, text: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "?InsertChars@ImGuiInputTextCallbackData@@QEAAXHPEBD0@Z";
    InsertChars :: (this: *InputTextCallbackData, pos: s32, text: string) #no_context {
        InsertChars(this, pos, text.data, text.data + text.count);
    }
}

InputTextFlags :: enum_flags s32 {
    None :: 0x0;
    CharsDecimal :: 0x1;
    CharsHexadecimal :: 0x2;
    CharsUppercase :: 0x4;
    CharsNoBlank :: 0x8;
    AutoSelectAll :: 0x10;
    EnterReturnsTrue :: 0x20;
    CallbackCompletion :: 0x40;
    CallbackHistory :: 0x80;
    CallbackAlways :: 0x100;
    CallbackCharFilter :: 0x200;
    AllowTabInput :: 0x400;
    CtrlEnterForNewLine :: 0x800;
    NoHorizontalScroll :: 0x1000;
    AlwaysOverwrite :: 0x2000;
    ReadOnly :: 0x4000;
    Password :: 0x8000;
    NoUndoRedo :: 0x10000;
    CharsScientific :: 0x20000;
    CallbackResize :: 0x40000;
    CallbackEdit :: 0x80000;
    EscapeClearsAll :: 0x100000;
    ImGuiInputTextFlags_None :: None;
    ImGuiInputTextFlags_CharsDecimal :: CharsDecimal;
    ImGuiInputTextFlags_CharsHexadecimal :: CharsHexadecimal;
    ImGuiInputTextFlags_CharsUppercase :: CharsUppercase;
    ImGuiInputTextFlags_CharsNoBlank :: CharsNoBlank;
    ImGuiInputTextFlags_AutoSelectAll :: AutoSelectAll;
    ImGuiInputTextFlags_EnterReturnsTrue :: EnterReturnsTrue;
    ImGuiInputTextFlags_CallbackCompletion :: CallbackCompletion;
    ImGuiInputTextFlags_CallbackHistory :: CallbackHistory;
    ImGuiInputTextFlags_CallbackAlways :: CallbackAlways;
    ImGuiInputTextFlags_CallbackCharFilter :: CallbackCharFilter;
    ImGuiInputTextFlags_AllowTabInput :: AllowTabInput;
    ImGuiInputTextFlags_CtrlEnterForNewLine :: CtrlEnterForNewLine;
    ImGuiInputTextFlags_NoHorizontalScroll :: NoHorizontalScroll;
    ImGuiInputTextFlags_AlwaysOverwrite :: AlwaysOverwrite;
    ImGuiInputTextFlags_ReadOnly :: ReadOnly;
    ImGuiInputTextFlags_Password :: Password;
    ImGuiInputTextFlags_NoUndoRedo :: NoUndoRedo;
    ImGuiInputTextFlags_CharsScientific :: CharsScientific;
    ImGuiInputTextFlags_CallbackResize :: CallbackResize;
    ImGuiInputTextFlags_CallbackEdit :: CallbackEdit;
    ImGuiInputTextFlags_EscapeClearsAll :: EscapeClearsAll;
}

IO :: struct {
    ConfigFlags_: ConfigFlags;
    BackendFlags_: BackendFlags;
    DisplaySize: ImVec2;
    DeltaTime: float;
    IniSavingRate: float;
    IniFilename: *u8;
    LogFilename: *u8;
    MouseDoubleClickTime: float;
    MouseDoubleClickMaxDist: float;
    MouseDragThreshold: float;
    KeyRepeatDelay: float;
    KeyRepeatRate: float;
    HoverDelayNormal: float;
    HoverDelayShort: float;
    UserData: *void;
    Fonts: *ImFontAtlas;
    FontGlobalScale: float;
    FontAllowUserScaling: bool;
    FontDefault: *ImFont;
    DisplayFramebufferScale: ImVec2;
    MouseDrawCursor: bool;
    ConfigMacOSXBehaviors: bool;
    ConfigInputTrickleEventQueue: bool;
    ConfigInputTextCursorBlink: bool;
    ConfigInputTextEnterKeepActive: bool;
    ConfigDragClickToInputText: bool;
    ConfigWindowsResizeFromEdges: bool;
    ConfigWindowsMoveFromTitleBarOnly: bool;
    ConfigMemoryCompactTimer: float;
    ConfigDebugBeginReturnValueOnce: bool;
    ConfigDebugBeginReturnValueLoop: bool;
    ConfigDebugIgnoreFocusLoss: bool;
    BackendPlatformName: *u8;
    BackendRendererName: *u8;
    BackendPlatformUserData: *void;
    BackendRendererUserData: *void;
    BackendLanguageUserData: *void;
    GetClipboardTextFn: #type (user_data: *void) -> *u8 #c_call;
    SetClipboardTextFn: #type (user_data: *void, text: *u8) -> void #c_call;
    ClipboardUserData: *void;
    SetPlatformImeDataFn: #type (viewport: *Viewport, data: *PlatformImeData) -> void #c_call;
    ImeWindowHandle: *void;
    AddKeyEvent :: (this: *IO, key: Key, down: bool) -> void #cpp_method #foreign imgui "?AddKeyEvent@ImGuiIO@@QEAAXW4ImGuiKey@@_N@Z";
    AddKeyAnalogEvent :: (this: *IO, key: Key, down: bool, v: float) -> void #cpp_method #foreign imgui "?AddKeyAnalogEvent@ImGuiIO@@QEAAXW4ImGuiKey@@_NM@Z";
    AddMousePosEvent :: (this: *IO, x: float, y: float) -> void #cpp_method #foreign imgui "?AddMousePosEvent@ImGuiIO@@QEAAXMM@Z";
    AddMouseButtonEvent :: (this: *IO, button: s32, down: bool) -> void #cpp_method #foreign imgui "?AddMouseButtonEvent@ImGuiIO@@QEAAXH_N@Z";
    AddMouseWheelEvent :: (this: *IO, wheel_x: float, wheel_y: float) -> void #cpp_method #foreign imgui "?AddMouseWheelEvent@ImGuiIO@@QEAAXMM@Z";
    AddMouseSourceEvent :: (this: *IO, source: MouseSource) -> void #cpp_method #foreign imgui "?AddMouseSourceEvent@ImGuiIO@@QEAAXW4ImGuiMouseSource@@@Z";
    AddFocusEvent :: (this: *IO, focused: bool) -> void #cpp_method #foreign imgui "?AddFocusEvent@ImGuiIO@@QEAAX_N@Z";
    AddInputCharacter :: (this: *IO, c: u32) -> void #cpp_method #foreign imgui "?AddInputCharacter@ImGuiIO@@QEAAXI@Z";
    AddInputCharacterUTF16 :: (this: *IO, c: ImWchar16) -> void #cpp_method #foreign imgui "?AddInputCharacterUTF16@ImGuiIO@@QEAAXG@Z";
    AddInputCharactersUTF8 :: (this: *IO, str: *u8) -> void #cpp_method #foreign imgui "?AddInputCharactersUTF8@ImGuiIO@@QEAAXPEBD@Z";
    SetKeyEventNativeData :: (this: *IO, key: Key, native_keycode: s32, native_scancode: s32, native_legacy_index: s32 = -1) -> void #cpp_method #foreign imgui "?SetKeyEventNativeData@ImGuiIO@@QEAAXW4ImGuiKey@@HHH@Z";
    SetAppAcceptingEvents :: (this: *IO, accepting_events: bool) -> void #cpp_method #foreign imgui "?SetAppAcceptingEvents@ImGuiIO@@QEAAX_N@Z";
    ClearInputCharacters :: (this: *IO) -> void #cpp_method #foreign imgui "?ClearInputCharacters@ImGuiIO@@QEAAXXZ";
    ClearInputKeys :: (this: *IO) -> void #cpp_method #foreign imgui "?ClearInputKeys@ImGuiIO@@QEAAXXZ";
    WantCaptureMouse: bool;
    WantCaptureKeyboard: bool;
    WantTextInput: bool;
    WantSetMousePos: bool;
    WantSaveIniSettings: bool;
    NavActive: bool;
    NavVisible: bool;
    Framerate: float;
    MetricsRenderVertices: s32;
    MetricsRenderIndices: s32;
    MetricsRenderWindows: s32;
    MetricsActiveWindows: s32;
    MetricsActiveAllocations: s32;
    MouseDelta: ImVec2;
    KeyMap: [652] s32;
    KeysDown: [652] bool;
    NavInputs: [16] float;
    Ctx: *ImGuiContext;
    MousePos: ImVec2;
    MouseDown: [5] bool;
    MouseWheel: float;
    MouseWheelH: float;
    MouseSource_: MouseSource;
    KeyCtrl: bool;
    KeyShift: bool;
    KeyAlt: bool;
    KeySuper: bool;
    KeyMods: KeyChord;
    KeysData: [652] KeyData;
    WantCaptureMouseUnlessPopupClose: bool;
    MousePosPrev: ImVec2;
    MouseClickedPos: [5] ImVec2;
    MouseClickedTime: [5] float64;
    MouseClicked: [5] bool;
    MouseDoubleClicked: [5] bool;
    MouseClickedCount: [5] ImU16;
    MouseClickedLastCount: [5] ImU16;
    MouseReleased: [5] bool;
    MouseDownOwned: [5] bool;
    MouseDownOwnedUnlessPopupClose: [5] bool;
    MouseWheelRequestAxisSwap: bool;
    MouseDownDuration: [5] float;
    MouseDownDurationPrev: [5] float;
    MouseDragMaxDistanceSqr: [5] float;
    PenPressure: float;
    AppFocusLost: bool;
    AppAcceptingEvents: bool;
    BackendUsingLegacyKeyArrays: ImS8;
    BackendUsingLegacyNavInputArray: bool;
    InputQueueSurrogate: ImWchar16;
    InputQueueCharacters: ImVector(ImWchar);
    Constructor :: (this: *IO) -> void #cpp_method #foreign imgui "??0ImGuiIO@@QEAA@XZ";
}

Key :: enum s32 {
    None :: 0;
    Tab :: 512;
    LeftArrow :: 513;
    RightArrow :: 514;
    UpArrow :: 515;
    DownArrow :: 516;
    PageUp :: 517;
    PageDown :: 518;
    Home :: 519;
    End :: 520;
    Insert :: 521;
    Delete :: 522;
    Backspace :: 523;
    Space :: 524;
    Enter :: 525;
    Escape :: 526;
    LeftCtrl :: 527;
    LeftShift :: 528;
    LeftAlt :: 529;
    LeftSuper :: 530;
    RightCtrl :: 531;
    RightShift :: 532;
    RightAlt :: 533;
    RightSuper :: 534;
    Menu :: 535;
    _0 :: 536;
    _1 :: 537;
    _2 :: 538;
    _3 :: 539;
    _4 :: 540;
    _5 :: 541;
    _6 :: 542;
    _7 :: 543;
    _8 :: 544;
    _9 :: 545;
    A :: 546;
    B :: 547;
    C :: 548;
    D :: 549;
    E :: 550;
    F :: 551;
    G :: 552;
    H :: 553;
    I :: 554;
    J :: 555;
    K :: 556;
    L :: 557;
    M :: 558;
    N :: 559;
    O :: 560;
    P :: 561;
    Q :: 562;
    R :: 563;
    S :: 564;
    T :: 565;
    U :: 566;
    V :: 567;
    W :: 568;
    X :: 569;
    Y :: 570;
    Z :: 571;
    F1 :: 572;
    F2 :: 573;
    F3 :: 574;
    F4 :: 575;
    F5 :: 576;
    F6 :: 577;
    F7 :: 578;
    F8 :: 579;
    F9 :: 580;
    F10 :: 581;
    F11 :: 582;
    F12 :: 583;
    Apostrophe :: 584;
    Comma :: 585;
    Minus :: 586;
    Period :: 587;
    Slash :: 588;
    Semicolon :: 589;
    Equal :: 590;
    LeftBracket :: 591;
    Backslash :: 592;
    RightBracket :: 593;
    GraveAccent :: 594;
    CapsLock :: 595;
    ScrollLock :: 596;
    NumLock :: 597;
    PrintScreen :: 598;
    Pause :: 599;
    Keypad0 :: 600;
    Keypad1 :: 601;
    Keypad2 :: 602;
    Keypad3 :: 603;
    Keypad4 :: 604;
    Keypad5 :: 605;
    Keypad6 :: 606;
    Keypad7 :: 607;
    Keypad8 :: 608;
    Keypad9 :: 609;
    KeypadDecimal :: 610;
    KeypadDivide :: 611;
    KeypadMultiply :: 612;
    KeypadSubtract :: 613;
    KeypadAdd :: 614;
    KeypadEnter :: 615;
    KeypadEqual :: 616;
    GamepadStart :: 617;
    GamepadBack :: 618;
    GamepadFaceLeft :: 619;
    GamepadFaceRight :: 620;
    GamepadFaceUp :: 621;
    GamepadFaceDown :: 622;
    GamepadDpadLeft :: 623;
    GamepadDpadRight :: 624;
    GamepadDpadUp :: 625;
    GamepadDpadDown :: 626;
    GamepadL1 :: 627;
    GamepadR1 :: 628;
    GamepadL2 :: 629;
    GamepadR2 :: 630;
    GamepadL3 :: 631;
    GamepadR3 :: 632;
    GamepadLStickLeft :: 633;
    GamepadLStickRight :: 634;
    GamepadLStickUp :: 635;
    GamepadLStickDown :: 636;
    GamepadRStickLeft :: 637;
    GamepadRStickRight :: 638;
    GamepadRStickUp :: 639;
    GamepadRStickDown :: 640;
    MouseLeft :: 641;
    MouseRight :: 642;
    MouseMiddle :: 643;
    MouseX1 :: 644;
    MouseX2 :: 645;
    MouseWheelX :: 646;
    MouseWheelY :: 647;
    ReservedForModCtrl :: 648;
    ReservedForModShift :: 649;
    ReservedForModAlt :: 650;
    ReservedForModSuper :: 651;
    COUNT :: 652;
    Mod_None :: 0;
    Mod_Ctrl :: 4096;
    Mod_Shift :: 8192;
    Mod_Alt :: 16384;
    Mod_Super :: 32768;
    Mod_Shortcut :: 2048;
    Mod_Mask_ :: 63488;
    NamedKey_BEGIN :: 512;
    NamedKey_END :: 652;
    NamedKey_COUNT :: 140;
    KeysData_SIZE :: 652;
    KeysData_OFFSET :: 0;
    ModCtrl :: 4096;
    ModShift :: 8192;
    ModAlt :: 16384;
    ModSuper :: 32768;
    KeyPadEnter :: 615;
    ImGuiKey_None :: None;
    ImGuiKey_Tab :: Tab;
    ImGuiKey_LeftArrow :: LeftArrow;
    ImGuiKey_RightArrow :: RightArrow;
    ImGuiKey_UpArrow :: UpArrow;
    ImGuiKey_DownArrow :: DownArrow;
    ImGuiKey_PageUp :: PageUp;
    ImGuiKey_PageDown :: PageDown;
    ImGuiKey_Home :: Home;
    ImGuiKey_End :: End;
    ImGuiKey_Insert :: Insert;
    ImGuiKey_Delete :: Delete;
    ImGuiKey_Backspace :: Backspace;
    ImGuiKey_Space :: Space;
    ImGuiKey_Enter :: Enter;
    ImGuiKey_Escape :: Escape;
    ImGuiKey_LeftCtrl :: LeftCtrl;
    ImGuiKey_LeftShift :: LeftShift;
    ImGuiKey_LeftAlt :: LeftAlt;
    ImGuiKey_LeftSuper :: LeftSuper;
    ImGuiKey_RightCtrl :: RightCtrl;
    ImGuiKey_RightShift :: RightShift;
    ImGuiKey_RightAlt :: RightAlt;
    ImGuiKey_RightSuper :: RightSuper;
    ImGuiKey_Menu :: Menu;
    ImGuiKey_0 :: _0;
    ImGuiKey_1 :: _1;
    ImGuiKey_2 :: _2;
    ImGuiKey_3 :: _3;
    ImGuiKey_4 :: _4;
    ImGuiKey_5 :: _5;
    ImGuiKey_6 :: _6;
    ImGuiKey_7 :: _7;
    ImGuiKey_8 :: _8;
    ImGuiKey_9 :: _9;
    ImGuiKey_A :: A;
    ImGuiKey_B :: B;
    ImGuiKey_C :: C;
    ImGuiKey_D :: D;
    ImGuiKey_E :: E;
    ImGuiKey_F :: F;
    ImGuiKey_G :: G;
    ImGuiKey_H :: H;
    ImGuiKey_I :: I;
    ImGuiKey_J :: J;
    ImGuiKey_K :: K;
    ImGuiKey_L :: L;
    ImGuiKey_M :: M;
    ImGuiKey_N :: N;
    ImGuiKey_O :: O;
    ImGuiKey_P :: P;
    ImGuiKey_Q :: Q;
    ImGuiKey_R :: R;
    ImGuiKey_S :: S;
    ImGuiKey_T :: T;
    ImGuiKey_U :: U;
    ImGuiKey_V :: V;
    ImGuiKey_W :: W;
    ImGuiKey_X :: X;
    ImGuiKey_Y :: Y;
    ImGuiKey_Z :: Z;
    ImGuiKey_F1 :: F1;
    ImGuiKey_F2 :: F2;
    ImGuiKey_F3 :: F3;
    ImGuiKey_F4 :: F4;
    ImGuiKey_F5 :: F5;
    ImGuiKey_F6 :: F6;
    ImGuiKey_F7 :: F7;
    ImGuiKey_F8 :: F8;
    ImGuiKey_F9 :: F9;
    ImGuiKey_F10 :: F10;
    ImGuiKey_F11 :: F11;
    ImGuiKey_F12 :: F12;
    ImGuiKey_Apostrophe :: Apostrophe;
    ImGuiKey_Comma :: Comma;
    ImGuiKey_Minus :: Minus;
    ImGuiKey_Period :: Period;
    ImGuiKey_Slash :: Slash;
    ImGuiKey_Semicolon :: Semicolon;
    ImGuiKey_Equal :: Equal;
    ImGuiKey_LeftBracket :: LeftBracket;
    ImGuiKey_Backslash :: Backslash;
    ImGuiKey_RightBracket :: RightBracket;
    ImGuiKey_GraveAccent :: GraveAccent;
    ImGuiKey_CapsLock :: CapsLock;
    ImGuiKey_ScrollLock :: ScrollLock;
    ImGuiKey_NumLock :: NumLock;
    ImGuiKey_PrintScreen :: PrintScreen;
    ImGuiKey_Pause :: Pause;
    ImGuiKey_Keypad0 :: Keypad0;
    ImGuiKey_Keypad1 :: Keypad1;
    ImGuiKey_Keypad2 :: Keypad2;
    ImGuiKey_Keypad3 :: Keypad3;
    ImGuiKey_Keypad4 :: Keypad4;
    ImGuiKey_Keypad5 :: Keypad5;
    ImGuiKey_Keypad6 :: Keypad6;
    ImGuiKey_Keypad7 :: Keypad7;
    ImGuiKey_Keypad8 :: Keypad8;
    ImGuiKey_Keypad9 :: Keypad9;
    ImGuiKey_KeypadDecimal :: KeypadDecimal;
    ImGuiKey_KeypadDivide :: KeypadDivide;
    ImGuiKey_KeypadMultiply :: KeypadMultiply;
    ImGuiKey_KeypadSubtract :: KeypadSubtract;
    ImGuiKey_KeypadAdd :: KeypadAdd;
    ImGuiKey_KeypadEnter :: KeypadEnter;
    ImGuiKey_KeypadEqual :: KeypadEqual;
    ImGuiKey_GamepadStart :: GamepadStart;
    ImGuiKey_GamepadBack :: GamepadBack;
    ImGuiKey_GamepadFaceLeft :: GamepadFaceLeft;
    ImGuiKey_GamepadFaceRight :: GamepadFaceRight;
    ImGuiKey_GamepadFaceUp :: GamepadFaceUp;
    ImGuiKey_GamepadFaceDown :: GamepadFaceDown;
    ImGuiKey_GamepadDpadLeft :: GamepadDpadLeft;
    ImGuiKey_GamepadDpadRight :: GamepadDpadRight;
    ImGuiKey_GamepadDpadUp :: GamepadDpadUp;
    ImGuiKey_GamepadDpadDown :: GamepadDpadDown;
    ImGuiKey_GamepadL1 :: GamepadL1;
    ImGuiKey_GamepadR1 :: GamepadR1;
    ImGuiKey_GamepadL2 :: GamepadL2;
    ImGuiKey_GamepadR2 :: GamepadR2;
    ImGuiKey_GamepadL3 :: GamepadL3;
    ImGuiKey_GamepadR3 :: GamepadR3;
    ImGuiKey_GamepadLStickLeft :: GamepadLStickLeft;
    ImGuiKey_GamepadLStickRight :: GamepadLStickRight;
    ImGuiKey_GamepadLStickUp :: GamepadLStickUp;
    ImGuiKey_GamepadLStickDown :: GamepadLStickDown;
    ImGuiKey_GamepadRStickLeft :: GamepadRStickLeft;
    ImGuiKey_GamepadRStickRight :: GamepadRStickRight;
    ImGuiKey_GamepadRStickUp :: GamepadRStickUp;
    ImGuiKey_GamepadRStickDown :: GamepadRStickDown;
    ImGuiKey_MouseLeft :: MouseLeft;
    ImGuiKey_MouseRight :: MouseRight;
    ImGuiKey_MouseMiddle :: MouseMiddle;
    ImGuiKey_MouseX1 :: MouseX1;
    ImGuiKey_MouseX2 :: MouseX2;
    ImGuiKey_MouseWheelX :: MouseWheelX;
    ImGuiKey_MouseWheelY :: MouseWheelY;
    ImGuiKey_ReservedForModCtrl :: ReservedForModCtrl;
    ImGuiKey_ReservedForModShift :: ReservedForModShift;
    ImGuiKey_ReservedForModAlt :: ReservedForModAlt;
    ImGuiKey_ReservedForModSuper :: ReservedForModSuper;
    ImGuiKey_COUNT :: COUNT;
    ImGuiMod_None :: Mod_None;
    ImGuiMod_Ctrl :: Mod_Ctrl;
    ImGuiMod_Shift :: Mod_Shift;
    ImGuiMod_Alt :: Mod_Alt;
    ImGuiMod_Super :: Mod_Super;
    ImGuiMod_Shortcut :: Mod_Shortcut;
    ImGuiMod_Mask_ :: Mod_Mask_;
    ImGuiKey_NamedKey_BEGIN :: NamedKey_BEGIN;
    ImGuiKey_NamedKey_END :: NamedKey_END;
    ImGuiKey_NamedKey_COUNT :: NamedKey_COUNT;
    ImGuiKey_KeysData_SIZE :: KeysData_SIZE;
    ImGuiKey_KeysData_OFFSET :: KeysData_OFFSET;
    ImGuiKey_ModCtrl :: ModCtrl;
    ImGuiKey_ModShift :: ModShift;
    ImGuiKey_ModAlt :: ModAlt;
    ImGuiKey_ModSuper :: ModSuper;
    ImGuiKey_KeyPadEnter :: KeyPadEnter;
}

KeyData :: struct {
    Down: bool;
    DownDuration: float;
    DownDurationPrev: float;
    AnalogValue: float;
}

ListClipper :: struct {
    Ctx: *ImGuiContext;
    DisplayStart: s32;
    DisplayEnd: s32;
    ItemsCount: s32;
    ItemsHeight: float;
    StartPosY: float;
    TempData: *void;
    Constructor :: (this: *ListClipper) -> void #cpp_method #foreign imgui "??0ImGuiListClipper@@QEAA@XZ";
    Destructor :: (this: *ListClipper) -> void #cpp_method #foreign imgui "??1ImGuiListClipper@@QEAA@XZ";
    Begin :: (this: *ListClipper, items_count: s32, items_height: float = -1.0) -> void #cpp_method #foreign imgui "?Begin@ImGuiListClipper@@QEAAXHM@Z";
    End :: (this: *ListClipper) -> void #cpp_method #foreign imgui "?End@ImGuiListClipper@@QEAAXXZ";
    Step :: (this: *ListClipper) -> bool #cpp_method #foreign imgui "?Step@ImGuiListClipper@@QEAA_NXZ";
    IncludeRangeByIndices :: (this: *ListClipper, item_begin: s32, item_end: s32) -> void #cpp_method #foreign imgui "?IncludeRangeByIndices@ImGuiListClipper@@QEAAXHH@Z";
}

ModFlags :: enum_flags s32 {
    None :: 0x0;
    Ctrl :: 0x1000;
    Shift :: 0x2000;
    Alt :: 0x4000;
    Super :: 0x8000;
    ImGuiModFlags_None :: None;
    ImGuiModFlags_Ctrl :: Ctrl;
    ImGuiModFlags_Shift :: Shift;
    ImGuiModFlags_Alt :: Alt;
    ImGuiModFlags_Super :: Super;
}

MouseButton :: enum s32 {
    Left :: 0;
    Right :: 1;
    Middle :: 2;
    COUNT :: 5;
    ImGuiMouseButton_Left :: Left;
    ImGuiMouseButton_Right :: Right;
    ImGuiMouseButton_Middle :: Middle;
    ImGuiMouseButton_COUNT :: COUNT;
}

MouseCursor :: enum s32 {
    None :: -1;
    Arrow :: 0;
    TextInput :: 1;
    ResizeAll :: 2;
    ResizeNS :: 3;
    ResizeEW :: 4;
    ResizeNESW :: 5;
    ResizeNWSE :: 6;
    Hand :: 7;
    NotAllowed :: 8;
    COUNT :: 9;
    ImGuiMouseCursor_None :: None;
    ImGuiMouseCursor_Arrow :: Arrow;
    ImGuiMouseCursor_TextInput :: TextInput;
    ImGuiMouseCursor_ResizeAll :: ResizeAll;
    ImGuiMouseCursor_ResizeNS :: ResizeNS;
    ImGuiMouseCursor_ResizeEW :: ResizeEW;
    ImGuiMouseCursor_ResizeNESW :: ResizeNESW;
    ImGuiMouseCursor_ResizeNWSE :: ResizeNWSE;
    ImGuiMouseCursor_Hand :: Hand;
    ImGuiMouseCursor_NotAllowed :: NotAllowed;
    ImGuiMouseCursor_COUNT :: COUNT;
}

MouseSource :: enum s32 {
    Mouse :: 0;
    TouchScreen :: 1;
    Pen :: 2;
    COUNT :: 3;
    ImGuiMouseSource_Mouse :: Mouse;
    ImGuiMouseSource_TouchScreen :: TouchScreen;
    ImGuiMouseSource_Pen :: Pen;
    ImGuiMouseSource_COUNT :: COUNT;
}

NavInput :: enum s32 {
    Activate :: 0;
    Cancel :: 1;
    Input :: 2;
    Menu :: 3;
    DpadLeft :: 4;
    DpadRight :: 5;
    DpadUp :: 6;
    DpadDown :: 7;
    LStickLeft :: 8;
    LStickRight :: 9;
    LStickUp :: 10;
    LStickDown :: 11;
    FocusPrev :: 12;
    FocusNext :: 13;
    TweakSlow :: 14;
    TweakFast :: 15;
    COUNT :: 16;
    ImGuiNavInput_Activate :: Activate;
    ImGuiNavInput_Cancel :: Cancel;
    ImGuiNavInput_Input :: Input;
    ImGuiNavInput_Menu :: Menu;
    ImGuiNavInput_DpadLeft :: DpadLeft;
    ImGuiNavInput_DpadRight :: DpadRight;
    ImGuiNavInput_DpadUp :: DpadUp;
    ImGuiNavInput_DpadDown :: DpadDown;
    ImGuiNavInput_LStickLeft :: LStickLeft;
    ImGuiNavInput_LStickRight :: LStickRight;
    ImGuiNavInput_LStickUp :: LStickUp;
    ImGuiNavInput_LStickDown :: LStickDown;
    ImGuiNavInput_FocusPrev :: FocusPrev;
    ImGuiNavInput_FocusNext :: FocusNext;
    ImGuiNavInput_TweakSlow :: TweakSlow;
    ImGuiNavInput_TweakFast :: TweakFast;
    ImGuiNavInput_COUNT :: COUNT;
}

OnceUponAFrame :: struct {
    RefFrame: s32;
}

Payload :: struct {
    Data: *void;
    DataSize: s32;
    SourceId: ID;
    SourceParentId: ID;
    DataFrameCount: s32;
    DataType: [33] u8;
    Preview: bool;
    Delivery: bool;
}

PlatformImeData :: struct {
    WantVisible: bool;
    InputPos: ImVec2;
    InputLineHeight: float;
}

PopupFlags :: enum_flags s32 {
    None :: 0x0;
    MouseButtonLeft :: 0x0;
    MouseButtonRight :: 0x1;
    MouseButtonMiddle :: 0x2;
    MouseButtonMask_ :: 0x1f;
    MouseButtonDefault_ :: 0x1;
    NoOpenOverExistingPopup :: 0x20;
    NoOpenOverItems :: 0x40;
    AnyPopupId :: 0x80;
    AnyPopupLevel :: 0x100;
    AnyPopup :: 0x180;
    ImGuiPopupFlags_None :: None;
    ImGuiPopupFlags_MouseButtonLeft :: MouseButtonLeft;
    ImGuiPopupFlags_MouseButtonRight :: MouseButtonRight;
    ImGuiPopupFlags_MouseButtonMiddle :: MouseButtonMiddle;
    ImGuiPopupFlags_MouseButtonMask_ :: MouseButtonMask_;
    ImGuiPopupFlags_MouseButtonDefault_ :: MouseButtonDefault_;
    ImGuiPopupFlags_NoOpenOverExistingPopup :: NoOpenOverExistingPopup;
    ImGuiPopupFlags_NoOpenOverItems :: NoOpenOverItems;
    ImGuiPopupFlags_AnyPopupId :: AnyPopupId;
    ImGuiPopupFlags_AnyPopupLevel :: AnyPopupLevel;
    ImGuiPopupFlags_AnyPopup :: AnyPopup;
}

SelectableFlags :: enum_flags s32 {
    None :: 0x0;
    DontClosePopups :: 0x1;
    SpanAllColumns :: 0x2;
    AllowDoubleClick :: 0x4;
    Disabled :: 0x8;
    AllowItemOverlap :: 0x10;
    ImGuiSelectableFlags_None :: None;
    ImGuiSelectableFlags_DontClosePopups :: DontClosePopups;
    ImGuiSelectableFlags_SpanAllColumns :: SpanAllColumns;
    ImGuiSelectableFlags_AllowDoubleClick :: AllowDoubleClick;
    ImGuiSelectableFlags_Disabled :: Disabled;
    ImGuiSelectableFlags_AllowItemOverlap :: AllowItemOverlap;
}

SizeCallbackData :: struct {
    UserData: *void;
    Pos: ImVec2;
    CurrentSize: ImVec2;
    DesiredSize: ImVec2;
}

SliderFlags :: enum_flags s32 {
    None :: 0x0;
    AlwaysClamp :: 0x10;
    Logarithmic :: 0x20;
    NoRoundToFormat :: 0x40;
    NoInput :: 0x80;
    InvalidMask_ :: 0x7000000f;
    ImGuiSliderFlags_None :: None;
    ImGuiSliderFlags_AlwaysClamp :: AlwaysClamp;
    ImGuiSliderFlags_Logarithmic :: Logarithmic;
    ImGuiSliderFlags_NoRoundToFormat :: NoRoundToFormat;
    ImGuiSliderFlags_NoInput :: NoInput;
    ImGuiSliderFlags_InvalidMask_ :: InvalidMask_;
}

SortDirection :: enum s32 {
    None :: 0;
    Ascending :: 1;
    Descending :: 2;
    ImGuiSortDirection_None :: None;
    ImGuiSortDirection_Ascending :: Ascending;
    ImGuiSortDirection_Descending :: Descending;
}

Storage :: struct {
    StoragePair :: struct {
        key: ID;
        val_p: *void;
        #overlay (val_p) val_i: s32;
        #overlay (val_p) val_f: float;
    };
    Data: ImVector(StoragePair);
    GetInt :: (this: *Storage, key: ID, default_val: s32 = 0) -> s32 #cpp_method #foreign imgui "?GetInt@ImGuiStorage@@QEBAHIH@Z";
    SetInt :: (this: *Storage, key: ID, val: s32) -> void #cpp_method #foreign imgui "?SetInt@ImGuiStorage@@QEAAXIH@Z";
    GetBool :: (this: *Storage, key: ID, default_val := false) -> bool #cpp_method #foreign imgui "?GetBool@ImGuiStorage@@QEBA_NI_N@Z";
    SetBool :: (this: *Storage, key: ID, val: bool) -> void #cpp_method #foreign imgui "?SetBool@ImGuiStorage@@QEAAXI_N@Z";
    GetFloat :: (this: *Storage, key: ID, default_val: float = 0.0) -> float #cpp_method #foreign imgui "?GetFloat@ImGuiStorage@@QEBAMIM@Z";
    SetFloat :: (this: *Storage, key: ID, val: float) -> void #cpp_method #foreign imgui "?SetFloat@ImGuiStorage@@QEAAXIM@Z";
    GetVoidPtr :: (this: *Storage, key: ID) -> *void #cpp_method #foreign imgui "?GetVoidPtr@ImGuiStorage@@QEBAPEAXI@Z";
    SetVoidPtr :: (this: *Storage, key: ID, val: *void) -> void #cpp_method #foreign imgui "?SetVoidPtr@ImGuiStorage@@QEAAXIPEAX@Z";
    GetIntRef :: (this: *Storage, key: ID, default_val: s32 = 0) -> *s32 #cpp_method #foreign imgui "?GetIntRef@ImGuiStorage@@QEAAPEAHIH@Z";
    GetBoolRef :: (this: *Storage, key: ID, default_val := false) -> *bool #cpp_method #foreign imgui "?GetBoolRef@ImGuiStorage@@QEAAPEA_NI_N@Z";
    GetFloatRef :: (this: *Storage, key: ID, default_val: float = 0.0) -> *float #cpp_method #foreign imgui "?GetFloatRef@ImGuiStorage@@QEAAPEAMIM@Z";
    GetVoidPtrRef :: (this: *Storage, key: ID, default_val: *void = null) -> **void #cpp_method #foreign imgui "?GetVoidPtrRef@ImGuiStorage@@QEAAPEAPEAXIPEAX@Z";
    SetAllInt :: (this: *Storage, val: s32) -> void #cpp_method #foreign imgui "?SetAllInt@ImGuiStorage@@QEAAXH@Z";
    BuildSortByKey :: (this: *Storage) -> void #cpp_method #foreign imgui "?BuildSortByKey@ImGuiStorage@@QEAAXXZ";
}

Style :: struct {
    Alpha: float;
    DisabledAlpha: float;
    WindowPadding: ImVec2;
    WindowRounding: float;
    WindowBorderSize: float;
    WindowMinSize: ImVec2;
    WindowTitleAlign: ImVec2;
    WindowMenuButtonPosition: Dir;
    ChildRounding: float;
    ChildBorderSize: float;
    PopupRounding: float;
    PopupBorderSize: float;
    FramePadding: ImVec2;
    FrameRounding: float;
    FrameBorderSize: float;
    ItemSpacing: ImVec2;
    ItemInnerSpacing: ImVec2;
    CellPadding: ImVec2;
    TouchExtraPadding: ImVec2;
    IndentSpacing: float;
    ColumnsMinSpacing: float;
    ScrollbarSize: float;
    ScrollbarRounding: float;
    GrabMinSize: float;
    GrabRounding: float;
    LogSliderDeadzone: float;
    TabRounding: float;
    TabBorderSize: float;
    TabMinWidthForCloseButton: float;
    ColorButtonPosition: Dir;
    ButtonTextAlign: ImVec2;
    SelectableTextAlign: ImVec2;
    SeparatorTextBorderSize: float;
    SeparatorTextAlign: ImVec2;
    SeparatorTextPadding: ImVec2;
    DisplayWindowPadding: ImVec2;
    DisplaySafeAreaPadding: ImVec2;
    MouseCursorScale: float;
    AntiAliasedLines: bool;
    AntiAliasedLinesUseTex: bool;
    AntiAliasedFill: bool;
    CurveTessellationTol: float;
    CircleTessellationMaxError: float;
    Colors: [53] ImVec4;
    Constructor :: (this: *Style) -> void #cpp_method #foreign imgui "??0ImGuiStyle@@QEAA@XZ";
    ScaleAllSizes :: (this: *Style, scale_factor: float) -> void #cpp_method #foreign imgui "?ScaleAllSizes@ImGuiStyle@@QEAAXM@Z";
}

StyleVar :: enum s32 {
    Alpha :: 0;
    DisabledAlpha :: 1;
    WindowPadding :: 2;
    WindowRounding :: 3;
    WindowBorderSize :: 4;
    WindowMinSize :: 5;
    WindowTitleAlign :: 6;
    ChildRounding :: 7;
    ChildBorderSize :: 8;
    PopupRounding :: 9;
    PopupBorderSize :: 10;
    FramePadding :: 11;
    FrameRounding :: 12;
    FrameBorderSize :: 13;
    ItemSpacing :: 14;
    ItemInnerSpacing :: 15;
    IndentSpacing :: 16;
    CellPadding :: 17;
    ScrollbarSize :: 18;
    ScrollbarRounding :: 19;
    GrabMinSize :: 20;
    GrabRounding :: 21;
    TabRounding :: 22;
    ButtonTextAlign :: 23;
    SelectableTextAlign :: 24;
    SeparatorTextBorderSize :: 25;
    SeparatorTextAlign :: 26;
    SeparatorTextPadding :: 27;
    COUNT :: 28;
    ImGuiStyleVar_Alpha :: Alpha;
    ImGuiStyleVar_DisabledAlpha :: DisabledAlpha;
    ImGuiStyleVar_WindowPadding :: WindowPadding;
    ImGuiStyleVar_WindowRounding :: WindowRounding;
    ImGuiStyleVar_WindowBorderSize :: WindowBorderSize;
    ImGuiStyleVar_WindowMinSize :: WindowMinSize;
    ImGuiStyleVar_WindowTitleAlign :: WindowTitleAlign;
    ImGuiStyleVar_ChildRounding :: ChildRounding;
    ImGuiStyleVar_ChildBorderSize :: ChildBorderSize;
    ImGuiStyleVar_PopupRounding :: PopupRounding;
    ImGuiStyleVar_PopupBorderSize :: PopupBorderSize;
    ImGuiStyleVar_FramePadding :: FramePadding;
    ImGuiStyleVar_FrameRounding :: FrameRounding;
    ImGuiStyleVar_FrameBorderSize :: FrameBorderSize;
    ImGuiStyleVar_ItemSpacing :: ItemSpacing;
    ImGuiStyleVar_ItemInnerSpacing :: ItemInnerSpacing;
    ImGuiStyleVar_IndentSpacing :: IndentSpacing;
    ImGuiStyleVar_CellPadding :: CellPadding;
    ImGuiStyleVar_ScrollbarSize :: ScrollbarSize;
    ImGuiStyleVar_ScrollbarRounding :: ScrollbarRounding;
    ImGuiStyleVar_GrabMinSize :: GrabMinSize;
    ImGuiStyleVar_GrabRounding :: GrabRounding;
    ImGuiStyleVar_TabRounding :: TabRounding;
    ImGuiStyleVar_ButtonTextAlign :: ButtonTextAlign;
    ImGuiStyleVar_SelectableTextAlign :: SelectableTextAlign;
    ImGuiStyleVar_SeparatorTextBorderSize :: SeparatorTextBorderSize;
    ImGuiStyleVar_SeparatorTextAlign :: SeparatorTextAlign;
    ImGuiStyleVar_SeparatorTextPadding :: SeparatorTextPadding;
    ImGuiStyleVar_COUNT :: COUNT;
}

TabBarFlags :: enum_flags s32 {
    None :: 0x0;
    Reorderable :: 0x1;
    AutoSelectNewTabs :: 0x2;
    TabListPopupButton :: 0x4;
    NoCloseWithMiddleMouseButton :: 0x8;
    NoTabListScrollingButtons :: 0x10;
    NoTooltip :: 0x20;
    FittingPolicyResizeDown :: 0x40;
    FittingPolicyScroll :: 0x80;
    FittingPolicyMask_ :: 0xc0;
    FittingPolicyDefault_ :: 0x40;
    ImGuiTabBarFlags_None :: None;
    ImGuiTabBarFlags_Reorderable :: Reorderable;
    ImGuiTabBarFlags_AutoSelectNewTabs :: AutoSelectNewTabs;
    ImGuiTabBarFlags_TabListPopupButton :: TabListPopupButton;
    ImGuiTabBarFlags_NoCloseWithMiddleMouseButton :: NoCloseWithMiddleMouseButton;
    ImGuiTabBarFlags_NoTabListScrollingButtons :: NoTabListScrollingButtons;
    ImGuiTabBarFlags_NoTooltip :: NoTooltip;
    ImGuiTabBarFlags_FittingPolicyResizeDown :: FittingPolicyResizeDown;
    ImGuiTabBarFlags_FittingPolicyScroll :: FittingPolicyScroll;
    ImGuiTabBarFlags_FittingPolicyMask_ :: FittingPolicyMask_;
    ImGuiTabBarFlags_FittingPolicyDefault_ :: FittingPolicyDefault_;
}

TabItemFlags :: enum_flags s32 {
    None :: 0x0;
    UnsavedDocument :: 0x1;
    SetSelected :: 0x2;
    NoCloseWithMiddleMouseButton :: 0x4;
    NoPushId :: 0x8;
    NoTooltip :: 0x10;
    NoReorder :: 0x20;
    Leading :: 0x40;
    Trailing :: 0x80;
    ImGuiTabItemFlags_None :: None;
    ImGuiTabItemFlags_UnsavedDocument :: UnsavedDocument;
    ImGuiTabItemFlags_SetSelected :: SetSelected;
    ImGuiTabItemFlags_NoCloseWithMiddleMouseButton :: NoCloseWithMiddleMouseButton;
    ImGuiTabItemFlags_NoPushId :: NoPushId;
    ImGuiTabItemFlags_NoTooltip :: NoTooltip;
    ImGuiTabItemFlags_NoReorder :: NoReorder;
    ImGuiTabItemFlags_Leading :: Leading;
    ImGuiTabItemFlags_Trailing :: Trailing;
}

TableBgTarget :: enum s32 {
    None :: 0;
    RowBg0 :: 1;
    RowBg1 :: 2;
    CellBg :: 3;
    ImGuiTableBgTarget_None :: None;
    ImGuiTableBgTarget_RowBg0 :: RowBg0;
    ImGuiTableBgTarget_RowBg1 :: RowBg1;
    ImGuiTableBgTarget_CellBg :: CellBg;
}

TableColumnFlags :: enum_flags s32 {
    None :: 0x0;
    Disabled :: 0x1;
    DefaultHide :: 0x2;
    DefaultSort :: 0x4;
    WidthStretch :: 0x8;
    WidthFixed :: 0x10;
    NoResize :: 0x20;
    NoReorder :: 0x40;
    NoHide :: 0x80;
    NoClip :: 0x100;
    NoSort :: 0x200;
    NoSortAscending :: 0x400;
    NoSortDescending :: 0x800;
    NoHeaderLabel :: 0x1000;
    NoHeaderWidth :: 0x2000;
    PreferSortAscending :: 0x4000;
    PreferSortDescending :: 0x8000;
    IndentEnable :: 0x10000;
    IndentDisable :: 0x20000;
    IsEnabled :: 0x1000000;
    IsVisible :: 0x2000000;
    IsSorted :: 0x4000000;
    IsHovered :: 0x8000000;
    WidthMask_ :: 0x18;
    IndentMask_ :: 0x30000;
    StatusMask_ :: 0xf000000;
    NoDirectResize_ :: 0x40000000;
    ImGuiTableColumnFlags_None :: None;
    ImGuiTableColumnFlags_Disabled :: Disabled;
    ImGuiTableColumnFlags_DefaultHide :: DefaultHide;
    ImGuiTableColumnFlags_DefaultSort :: DefaultSort;
    ImGuiTableColumnFlags_WidthStretch :: WidthStretch;
    ImGuiTableColumnFlags_WidthFixed :: WidthFixed;
    ImGuiTableColumnFlags_NoResize :: NoResize;
    ImGuiTableColumnFlags_NoReorder :: NoReorder;
    ImGuiTableColumnFlags_NoHide :: NoHide;
    ImGuiTableColumnFlags_NoClip :: NoClip;
    ImGuiTableColumnFlags_NoSort :: NoSort;
    ImGuiTableColumnFlags_NoSortAscending :: NoSortAscending;
    ImGuiTableColumnFlags_NoSortDescending :: NoSortDescending;
    ImGuiTableColumnFlags_NoHeaderLabel :: NoHeaderLabel;
    ImGuiTableColumnFlags_NoHeaderWidth :: NoHeaderWidth;
    ImGuiTableColumnFlags_PreferSortAscending :: PreferSortAscending;
    ImGuiTableColumnFlags_PreferSortDescending :: PreferSortDescending;
    ImGuiTableColumnFlags_IndentEnable :: IndentEnable;
    ImGuiTableColumnFlags_IndentDisable :: IndentDisable;
    ImGuiTableColumnFlags_IsEnabled :: IsEnabled;
    ImGuiTableColumnFlags_IsVisible :: IsVisible;
    ImGuiTableColumnFlags_IsSorted :: IsSorted;
    ImGuiTableColumnFlags_IsHovered :: IsHovered;
    ImGuiTableColumnFlags_WidthMask_ :: WidthMask_;
    ImGuiTableColumnFlags_IndentMask_ :: IndentMask_;
    ImGuiTableColumnFlags_StatusMask_ :: StatusMask_;
    ImGuiTableColumnFlags_NoDirectResize_ :: NoDirectResize_;
}

TableColumnSortSpecs :: struct {
    ColumnUserID: ID;
    ColumnIndex: ImS16;
    SortOrder: ImS16;
    SortDirection_: SortDirection;
}

TableFlags :: enum_flags s32 {
    None :: 0x0;
    Resizable :: 0x1;
    Reorderable :: 0x2;
    Hideable :: 0x4;
    Sortable :: 0x8;
    NoSavedSettings :: 0x10;
    ContextMenuInBody :: 0x20;
    RowBg :: 0x40;
    BordersInnerH :: 0x80;
    BordersOuterH :: 0x100;
    BordersInnerV :: 0x200;
    BordersOuterV :: 0x400;
    BordersH :: 0x180;
    BordersV :: 0x600;
    BordersInner :: 0x280;
    BordersOuter :: 0x500;
    Borders :: 0x780;
    NoBordersInBody :: 0x800;
    NoBordersInBodyUntilResize :: 0x1000;
    SizingFixedFit :: 0x2000;
    SizingFixedSame :: 0x4000;
    SizingStretchProp :: 0x6000;
    SizingStretchSame :: 0x8000;
    NoHostExtendX :: 0x10000;
    NoHostExtendY :: 0x20000;
    NoKeepColumnsVisible :: 0x40000;
    PreciseWidths :: 0x80000;
    NoClip :: 0x100000;
    PadOuterX :: 0x200000;
    NoPadOuterX :: 0x400000;
    NoPadInnerX :: 0x800000;
    ScrollX :: 0x1000000;
    ScrollY :: 0x2000000;
    SortMulti :: 0x4000000;
    SortTristate :: 0x8000000;
    SizingMask_ :: 0xe000;
    ImGuiTableFlags_None :: None;
    ImGuiTableFlags_Resizable :: Resizable;
    ImGuiTableFlags_Reorderable :: Reorderable;
    ImGuiTableFlags_Hideable :: Hideable;
    ImGuiTableFlags_Sortable :: Sortable;
    ImGuiTableFlags_NoSavedSettings :: NoSavedSettings;
    ImGuiTableFlags_ContextMenuInBody :: ContextMenuInBody;
    ImGuiTableFlags_RowBg :: RowBg;
    ImGuiTableFlags_BordersInnerH :: BordersInnerH;
    ImGuiTableFlags_BordersOuterH :: BordersOuterH;
    ImGuiTableFlags_BordersInnerV :: BordersInnerV;
    ImGuiTableFlags_BordersOuterV :: BordersOuterV;
    ImGuiTableFlags_BordersH :: BordersH;
    ImGuiTableFlags_BordersV :: BordersV;
    ImGuiTableFlags_BordersInner :: BordersInner;
    ImGuiTableFlags_BordersOuter :: BordersOuter;
    ImGuiTableFlags_Borders :: Borders;
    ImGuiTableFlags_NoBordersInBody :: NoBordersInBody;
    ImGuiTableFlags_NoBordersInBodyUntilResize :: NoBordersInBodyUntilResize;
    ImGuiTableFlags_SizingFixedFit :: SizingFixedFit;
    ImGuiTableFlags_SizingFixedSame :: SizingFixedSame;
    ImGuiTableFlags_SizingStretchProp :: SizingStretchProp;
    ImGuiTableFlags_SizingStretchSame :: SizingStretchSame;
    ImGuiTableFlags_NoHostExtendX :: NoHostExtendX;
    ImGuiTableFlags_NoHostExtendY :: NoHostExtendY;
    ImGuiTableFlags_NoKeepColumnsVisible :: NoKeepColumnsVisible;
    ImGuiTableFlags_PreciseWidths :: PreciseWidths;
    ImGuiTableFlags_NoClip :: NoClip;
    ImGuiTableFlags_PadOuterX :: PadOuterX;
    ImGuiTableFlags_NoPadOuterX :: NoPadOuterX;
    ImGuiTableFlags_NoPadInnerX :: NoPadInnerX;
    ImGuiTableFlags_ScrollX :: ScrollX;
    ImGuiTableFlags_ScrollY :: ScrollY;
    ImGuiTableFlags_SortMulti :: SortMulti;
    ImGuiTableFlags_SortTristate :: SortTristate;
    ImGuiTableFlags_SizingMask_ :: SizingMask_;
}

TableRowFlags :: enum_flags s32 {
    None :: 0x0;
    Headers :: 0x1;
    ImGuiTableRowFlags_None :: None;
    ImGuiTableRowFlags_Headers :: Headers;
}

TableSortSpecs :: struct {
    Specs: *TableColumnSortSpecs;
    SpecsCount: s32;
    SpecsDirty: bool;
}

TextFilter :: struct {
    Constructor :: (this: *TextFilter, default_filter: *u8 = "") -> void #cpp_method #foreign imgui "??0ImGuiTextFilter@@QEAA@PEBD@Z";
    Draw :: (this: *TextFilter, label: *u8 = "Filter (inc,-exc)", width: float = 0.0) -> bool #cpp_method #foreign imgui "?Draw@ImGuiTextFilter@@QEAA_NPEBDM@Z";
    PassFilter :: (this: *TextFilter, text: *u8, text_end: *u8 = null) -> bool #cpp_method #foreign imgui "?PassFilter@ImGuiTextFilter@@QEBA_NPEBD0@Z";
    PassFilter :: (this: *TextFilter, text: string) -> bool #no_context {
        return PassFilter(this, text.data, text.data + text.count);
    }
    Build :: (this: *TextFilter) -> void #cpp_method #foreign imgui "?Build@ImGuiTextFilter@@QEAAXXZ";
    TextRange :: struct {
        b: *u8;
        e: *u8;
        split :: (this: *TextRange, separator: u8, out: *ImVector(TextRange)) -> void #cpp_method #foreign imgui "?split@ImGuiTextRange@ImGuiTextFilter@@QEBAXDPEAU?$ImVector@UImGuiTextRange@ImGuiTextFilter@@@@@Z";
    };
    InputBuf: [256] u8;
    Filters: ImVector(TextRange);
    CountGrep: s32;
}

TreeNodeFlags :: enum_flags s32 {
    None :: 0x0;
    Selected :: 0x1;
    Framed :: 0x2;
    AllowItemOverlap :: 0x4;
    NoTreePushOnOpen :: 0x8;
    NoAutoOpenOnLog :: 0x10;
    DefaultOpen :: 0x20;
    OpenOnDoubleClick :: 0x40;
    OpenOnArrow :: 0x80;
    Leaf :: 0x100;
    Bullet :: 0x200;
    FramePadding :: 0x400;
    SpanAvailWidth :: 0x800;
    SpanFullWidth :: 0x1000;
    NavLeftJumpsBackHere :: 0x2000;
    CollapsingHeader :: 0x1a;
    ImGuiTreeNodeFlags_None :: None;
    ImGuiTreeNodeFlags_Selected :: Selected;
    ImGuiTreeNodeFlags_Framed :: Framed;
    ImGuiTreeNodeFlags_AllowItemOverlap :: AllowItemOverlap;
    ImGuiTreeNodeFlags_NoTreePushOnOpen :: NoTreePushOnOpen;
    ImGuiTreeNodeFlags_NoAutoOpenOnLog :: NoAutoOpenOnLog;
    ImGuiTreeNodeFlags_DefaultOpen :: DefaultOpen;
    ImGuiTreeNodeFlags_OpenOnDoubleClick :: OpenOnDoubleClick;
    ImGuiTreeNodeFlags_OpenOnArrow :: OpenOnArrow;
    ImGuiTreeNodeFlags_Leaf :: Leaf;
    ImGuiTreeNodeFlags_Bullet :: Bullet;
    ImGuiTreeNodeFlags_FramePadding :: FramePadding;
    ImGuiTreeNodeFlags_SpanAvailWidth :: SpanAvailWidth;
    ImGuiTreeNodeFlags_SpanFullWidth :: SpanFullWidth;
    ImGuiTreeNodeFlags_NavLeftJumpsBackHere :: NavLeftJumpsBackHere;
    ImGuiTreeNodeFlags_CollapsingHeader :: CollapsingHeader;
}

Viewport :: struct {
    Flags: ViewportFlags;
    Pos: ImVec2;
    Size: ImVec2;
    WorkPos: ImVec2;
    WorkSize: ImVec2;
    PlatformHandleRaw: *void;
}

ViewportFlags :: enum_flags s32 {
    None :: 0x0;
    IsPlatformWindow :: 0x1;
    IsPlatformMonitor :: 0x2;
    OwnedByApp :: 0x4;
    ImGuiViewportFlags_None :: None;
    ImGuiViewportFlags_IsPlatformWindow :: IsPlatformWindow;
    ImGuiViewportFlags_IsPlatformMonitor :: IsPlatformMonitor;
    ImGuiViewportFlags_OwnedByApp :: OwnedByApp;
}

WindowFlags :: enum_flags s32 {
    None :: 0x0;
    NoTitleBar :: 0x1;
    NoResize :: 0x2;
    NoMove :: 0x4;
    NoScrollbar :: 0x8;
    NoScrollWithMouse :: 0x10;
    NoCollapse :: 0x20;
    AlwaysAutoResize :: 0x40;
    NoBackground :: 0x80;
    NoSavedSettings :: 0x100;
    NoMouseInputs :: 0x200;
    MenuBar :: 0x400;
    HorizontalScrollbar :: 0x800;
    NoFocusOnAppearing :: 0x1000;
    NoBringToFrontOnFocus :: 0x2000;
    AlwaysVerticalScrollbar :: 0x4000;
    AlwaysHorizontalScrollbar :: 0x8000;
    AlwaysUseWindowPadding :: 0x10000;
    NoNavInputs :: 0x40000;
    NoNavFocus :: 0x80000;
    UnsavedDocument :: 0x100000;
    NoNav :: 0xc0000;
    NoDecoration :: 0x2b;
    NoInputs :: 0xc0200;
    NavFlattened :: 0x800000;
    ChildWindow :: 0x1000000;
    Tooltip :: 0x2000000;
    Popup :: 0x4000000;
    Modal :: 0x8000000;
    ChildMenu :: 0x10000000;
    ImGuiWindowFlags_None :: None;
    ImGuiWindowFlags_NoTitleBar :: NoTitleBar;
    ImGuiWindowFlags_NoResize :: NoResize;
    ImGuiWindowFlags_NoMove :: NoMove;
    ImGuiWindowFlags_NoScrollbar :: NoScrollbar;
    ImGuiWindowFlags_NoScrollWithMouse :: NoScrollWithMouse;
    ImGuiWindowFlags_NoCollapse :: NoCollapse;
    ImGuiWindowFlags_AlwaysAutoResize :: AlwaysAutoResize;
    ImGuiWindowFlags_NoBackground :: NoBackground;
    ImGuiWindowFlags_NoSavedSettings :: NoSavedSettings;
    ImGuiWindowFlags_NoMouseInputs :: NoMouseInputs;
    ImGuiWindowFlags_MenuBar :: MenuBar;
    ImGuiWindowFlags_HorizontalScrollbar :: HorizontalScrollbar;
    ImGuiWindowFlags_NoFocusOnAppearing :: NoFocusOnAppearing;
    ImGuiWindowFlags_NoBringToFrontOnFocus :: NoBringToFrontOnFocus;
    ImGuiWindowFlags_AlwaysVerticalScrollbar :: AlwaysVerticalScrollbar;
    ImGuiWindowFlags_AlwaysHorizontalScrollbar :: AlwaysHorizontalScrollbar;
    ImGuiWindowFlags_AlwaysUseWindowPadding :: AlwaysUseWindowPadding;
    ImGuiWindowFlags_NoNavInputs :: NoNavInputs;
    ImGuiWindowFlags_NoNavFocus :: NoNavFocus;
    ImGuiWindowFlags_UnsavedDocument :: UnsavedDocument;
    ImGuiWindowFlags_NoNav :: NoNav;
    ImGuiWindowFlags_NoDecoration :: NoDecoration;
    ImGuiWindowFlags_NoInputs :: NoInputs;
    ImGuiWindowFlags_NavFlattened :: NavFlattened;
    ImGuiWindowFlags_ChildWindow :: ChildWindow;
    ImGuiWindowFlags_Tooltip :: Tooltip;
    ImGuiWindowFlags_Popup :: Popup;
    ImGuiWindowFlags_Modal :: Modal;
    ImGuiWindowFlags_ChildMenu :: ChildMenu;
}

InputTextCallback :: #type (data: *InputTextCallbackData) -> s32 #c_call;

MemAllocFunc :: #type (sz: u64, user_data: *void) -> *void #c_call;

MemFreeFunc :: #type (ptr: *void, user_data: *void) -> void #c_call;

SizeCallback :: #type (data: *SizeCallbackData) -> void #c_call;

ID :: u32;

IM_COL32_A_MASK :: 0xFF000000;

IM_COL32_A_SHIFT :: 24;

IM_COL32_B_SHIFT :: 16;

IM_COL32_G_SHIFT :: 8;

IM_COL32_R_SHIFT :: 0;

IM_DRAWLIST_TEX_LINES_WIDTH_MAX :: 63;

IM_UNICODE_CODEPOINT_INVALID :: 0xFFFD;

IM_UNICODE_CODEPOINT_MAX :: 0xFFFF;

ImDrawIdx :: u16;

IMGUI_PAYLOAD_TYPE_COLOR_3F :: "_COL3F";

IMGUI_PAYLOAD_TYPE_COLOR_4F :: "_COL4F";

IMGUI_VERSION :: "1.89.6";

IMGUI_VERSION_NUM :: 18960;

ImS16 :: s16;

ImS32 :: s32;

ImS64 :: s64;

ImS8 :: s8;

ImTextureID :: *void;

ImU16 :: u16;

ImU32 :: u32;

ImU64 :: u64;

ImU8 :: u8;

ImWchar :: ImWchar16;

ImWchar16 :: u16;

ImWchar32 :: u32;

KeyChord :: s32;

AcceptDragDropPayload :: (type: *u8, flags: DragDropFlags = .None) -> *Payload #foreign imgui "?AcceptDragDropPayload@ImGui@@YAPEBUImGuiPayload@@PEBDH@Z";

AlignTextToFramePadding :: () -> void #foreign imgui "?AlignTextToFramePadding@ImGui@@YAXXZ";

ArrowButton :: (str_id: *u8, dir: Dir) -> bool #foreign imgui "?ArrowButton@ImGui@@YA_NPEBDH@Z";

Begin :: (name: *u8, p_open: *bool = null, flags: WindowFlags = .None) -> bool #foreign imgui "?Begin@ImGui@@YA_NPEBDPEA_NH@Z";

BeginChild :: (str_id: *u8, size: *ImVec2, border := false, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginChild@ImGui@@YA_NPEBDAEBUImVec2@@_NH@Z";

BeginChild :: (id: ID, size: *ImVec2, border := false, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginChild@ImGui@@YA_NIAEBUImVec2@@_NH@Z";

BeginChildFrame :: (id: ID, size: *ImVec2, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginChildFrame@ImGui@@YA_NIAEBUImVec2@@H@Z";

BeginCombo :: (label: *u8, preview_value: *u8, flags: ComboFlags = .None) -> bool #foreign imgui "?BeginCombo@ImGui@@YA_NPEBD0H@Z";

BeginDisabled :: (disabled := true) -> void #foreign imgui "?BeginDisabled@ImGui@@YAX_N@Z";

BeginDragDropSource :: (flags: DragDropFlags = .None) -> bool #foreign imgui "?BeginDragDropSource@ImGui@@YA_NH@Z";

BeginDragDropTarget :: () -> bool #foreign imgui "?BeginDragDropTarget@ImGui@@YA_NXZ";

BeginGroup :: () -> void #foreign imgui "?BeginGroup@ImGui@@YAXXZ";

BeginListBox :: (label: *u8, size: *ImVec2) -> bool #foreign imgui "?BeginListBox@ImGui@@YA_NPEBDAEBUImVec2@@@Z";

BeginMainMenuBar :: () -> bool #foreign imgui "?BeginMainMenuBar@ImGui@@YA_NXZ";

BeginMenu :: (label: *u8, enabled := true) -> bool #foreign imgui "?BeginMenu@ImGui@@YA_NPEBD_N@Z";

BeginMenuBar :: () -> bool #foreign imgui "?BeginMenuBar@ImGui@@YA_NXZ";

BeginPopup :: (str_id: *u8, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginPopup@ImGui@@YA_NPEBDH@Z";

BeginPopupContextItem :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "?BeginPopupContextItem@ImGui@@YA_NPEBDH@Z";

BeginPopupContextVoid :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "?BeginPopupContextVoid@ImGui@@YA_NPEBDH@Z";

BeginPopupContextWindow :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "?BeginPopupContextWindow@ImGui@@YA_NPEBDH@Z";

BeginPopupModal :: (name: *u8, p_open: *bool = null, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginPopupModal@ImGui@@YA_NPEBDPEA_NH@Z";

BeginTabBar :: (str_id: *u8, flags: TabBarFlags = .None) -> bool #foreign imgui "?BeginTabBar@ImGui@@YA_NPEBDH@Z";

BeginTabItem :: (label: *u8, p_open: *bool = null, flags: TabItemFlags = .None) -> bool #foreign imgui "?BeginTabItem@ImGui@@YA_NPEBDPEA_NH@Z";

BeginTable :: (str_id: *u8, column: s32, flags: TableFlags = .None, outer_size: *ImVec2, inner_width: float = 0.0) -> bool #foreign imgui "?BeginTable@ImGui@@YA_NPEBDHHAEBUImVec2@@M@Z";

BeginTooltip :: () -> bool #foreign imgui "?BeginTooltip@ImGui@@YA_NXZ";

Bullet :: () -> void #foreign imgui "?Bullet@ImGui@@YAXXZ";

BulletText :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?BulletText@ImGui@@YAXPEBDZZ";

Button :: (label: *u8, size: *ImVec2) -> bool #foreign imgui "?Button@ImGui@@YA_NPEBDAEBUImVec2@@@Z";

CalcItemWidth :: () -> float #foreign imgui "?CalcItemWidth@ImGui@@YAMXZ";

CalcListClipping :: (items_count: s32, items_height: float, out_items_display_start: *s32, out_items_display_end: *s32) -> void #foreign imgui "?CalcListClipping@ImGui@@YAXHMPEAH0@Z";

CalcTextSize :: (text: *u8, text_end: *u8 = null, hide_text_after_double_hash := false, wrap_width: float = -1.0) -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?CalcTextSize@ImGui@@YA?AUImVec2@@PEBD0_NM@Z";

Checkbox :: (label: *u8, v: *bool) -> bool #foreign imgui "?Checkbox@ImGui@@YA_NPEBDPEA_N@Z";

CheckboxFlags :: (label: *u8, flags: *s32, flags_value: s32) -> bool #foreign imgui "?CheckboxFlags@ImGui@@YA_NPEBDPEAHH@Z";

CheckboxFlags :: (label: *u8, flags: *u32, flags_value: u32) -> bool #foreign imgui "?CheckboxFlags@ImGui@@YA_NPEBDPEAII@Z";

CloseCurrentPopup :: () -> void #foreign imgui "?CloseCurrentPopup@ImGui@@YAXXZ";

CollapsingHeader :: (label: *u8, flags: TreeNodeFlags = .None) -> bool #foreign imgui "?CollapsingHeader@ImGui@@YA_NPEBDH@Z";

CollapsingHeader :: (label: *u8, p_visible: *bool, flags: TreeNodeFlags = .None) -> bool #foreign imgui "?CollapsingHeader@ImGui@@YA_NPEBDPEA_NH@Z";

ColorButton :: (desc_id: *u8, col: *ImVec4, flags: ColorEditFlags = .None, size: *ImVec2) -> bool #foreign imgui "?ColorButton@ImGui@@YA_NPEBDAEBUImVec4@@HAEBUImVec2@@@Z";

ColorConvertFloat4ToU32 :: (in: *ImVec4) -> ImU32 #foreign imgui "?ColorConvertFloat4ToU32@ImGui@@YAIAEBUImVec4@@@Z";

ColorConvertHSVtoRGB :: (h: float, s: float, v: float, out_r: *float, out_g: *float, out_b: *float) -> void #foreign imgui "?ColorConvertHSVtoRGB@ImGui@@YAXMMMAEAM00@Z";

ColorConvertRGBtoHSV :: (r: float, g: float, b: float, out_h: *float, out_s: *float, out_v: *float) -> void #foreign imgui "?ColorConvertRGBtoHSV@ImGui@@YAXMMMAEAM00@Z";

ColorConvertU32ToFloat4 :: (in: ImU32) -> ImVec4 #cpp_return_type_is_non_pod #foreign imgui "?ColorConvertU32ToFloat4@ImGui@@YA?AUImVec4@@I@Z";

ColorEdit3 :: (label: *u8, col: *[3] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "?ColorEdit3@ImGui@@YA_NPEBDQEAMH@Z";

ColorEdit4 :: (label: *u8, col: *[4] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "?ColorEdit4@ImGui@@YA_NPEBDQEAMH@Z";

ColorPicker3 :: (label: *u8, col: *[3] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "?ColorPicker3@ImGui@@YA_NPEBDQEAMH@Z";

ColorPicker4 :: (label: *u8, col: *[4] float, flags: ColorEditFlags = .None, ref_col: *float = null) -> bool #foreign imgui "?ColorPicker4@ImGui@@YA_NPEBDQEAMHPEBM@Z";

Columns :: (count: s32 = 1, id: *u8 = null, border := true) -> void #foreign imgui "?Columns@ImGui@@YAXHPEBD_N@Z";

Combo :: (label: *u8, current_item: *s32, items: **u8, items_count: s32, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "?Combo@ImGui@@YA_NPEBDPEAHQEBQEBDHH@Z";

Combo :: (label: *u8, current_item: *s32, items_separated_by_zeros: *u8, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "?Combo@ImGui@@YA_NPEBDPEAH0H@Z";

Combo :: (label: *u8, current_item: *s32, items_getter: #type (data: *void, idx: s32, out_text: **u8) -> bool #c_call, data: *void, items_count: s32, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "?Combo@ImGui@@YA_NPEBDPEAHP6A_NPEAXHPEAPEBD@Z2HH@Z";

CreateContext :: (shared_font_atlas: *ImFontAtlas = null) -> *ImGuiContext #foreign imgui "?CreateContext@ImGui@@YAPEAUImGuiContext@@PEAUImFontAtlas@@@Z";

DebugCheckVersionAndDataLayout :: (version_str: *u8, sz_io: u64, sz_style: u64, sz_vec2: u64, sz_vec4: u64, sz_drawvert: u64, sz_drawidx: u64) -> bool #foreign imgui "?DebugCheckVersionAndDataLayout@ImGui@@YA_NPEBD_K11111@Z";

DebugTextEncoding :: (text: *u8) -> void #foreign imgui "?DebugTextEncoding@ImGui@@YAXPEBD@Z";

DestroyContext :: (ctx: *ImGuiContext = null) -> void #foreign imgui "?DestroyContext@ImGui@@YAXPEAUImGuiContext@@@Z";

DragFloat :: (label: *u8, v: *float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat@ImGui@@YA_NPEBDPEAMMMM0H@Z";

DragFloat2 :: (label: *u8, v: *[2] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat2@ImGui@@YA_NPEBDQEAMMMM0H@Z";

DragFloat3 :: (label: *u8, v: *[3] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat3@ImGui@@YA_NPEBDQEAMMMM0H@Z";

DragFloat4 :: (label: *u8, v: *[4] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat4@ImGui@@YA_NPEBDQEAMMMM0H@Z";

DragFloatRange2 :: (label: *u8, v_current_min: *float, v_current_max: *float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", format_max: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloatRange2@ImGui@@YA_NPEBDPEAM1MMM00H@Z";

DragInt :: (label: *u8, v: *s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt@ImGui@@YA_NPEBDPEAHMHH0H@Z";

DragInt2 :: (label: *u8, v: *[2] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt2@ImGui@@YA_NPEBDQEAHMHH0H@Z";

DragInt3 :: (label: *u8, v: *[3] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt3@ImGui@@YA_NPEBDQEAHMHH0H@Z";

DragInt4 :: (label: *u8, v: *[4] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt4@ImGui@@YA_NPEBDQEAHMHH0H@Z";

DragIntRange2 :: (label: *u8, v_current_min: *s32, v_current_max: *s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", format_max: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragIntRange2@ImGui@@YA_NPEBDPEAH1MHH00H@Z";

DragScalar :: (label: *u8, data_type: DataType, p_data: *void, v_speed: float = 1.0, p_min: *void = null, p_max: *void = null, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragScalar@ImGui@@YA_NPEBDHPEAXMPEBX20H@Z";

DragScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, v_speed: float = 1.0, p_min: *void = null, p_max: *void = null, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragScalarN@ImGui@@YA_NPEBDHPEAXHMPEBX20H@Z";

Dummy :: (size: *ImVec2) -> void #foreign imgui "?Dummy@ImGui@@YAXAEBUImVec2@@@Z";

End :: () -> void #foreign imgui "?End@ImGui@@YAXXZ";

EndChild :: () -> void #foreign imgui "?EndChild@ImGui@@YAXXZ";

EndChildFrame :: () -> void #foreign imgui "?EndChildFrame@ImGui@@YAXXZ";

EndCombo :: () -> void #foreign imgui "?EndCombo@ImGui@@YAXXZ";

EndDisabled :: () -> void #foreign imgui "?EndDisabled@ImGui@@YAXXZ";

EndDragDropSource :: () -> void #foreign imgui "?EndDragDropSource@ImGui@@YAXXZ";

EndDragDropTarget :: () -> void #foreign imgui "?EndDragDropTarget@ImGui@@YAXXZ";

EndFrame :: () -> void #foreign imgui "?EndFrame@ImGui@@YAXXZ";

EndGroup :: () -> void #foreign imgui "?EndGroup@ImGui@@YAXXZ";

EndListBox :: () -> void #foreign imgui "?EndListBox@ImGui@@YAXXZ";

EndMainMenuBar :: () -> void #foreign imgui "?EndMainMenuBar@ImGui@@YAXXZ";

EndMenu :: () -> void #foreign imgui "?EndMenu@ImGui@@YAXXZ";

EndMenuBar :: () -> void #foreign imgui "?EndMenuBar@ImGui@@YAXXZ";

EndPopup :: () -> void #foreign imgui "?EndPopup@ImGui@@YAXXZ";

EndTabBar :: () -> void #foreign imgui "?EndTabBar@ImGui@@YAXXZ";

EndTabItem :: () -> void #foreign imgui "?EndTabItem@ImGui@@YAXXZ";

EndTable :: () -> void #foreign imgui "?EndTable@ImGui@@YAXXZ";

EndTooltip :: () -> void #foreign imgui "?EndTooltip@ImGui@@YAXXZ";

GetAllocatorFunctions :: (p_alloc_func: *MemAllocFunc, p_free_func: *MemFreeFunc, p_user_data: **void) -> void #foreign imgui "?GetAllocatorFunctions@ImGui@@YAXPEAP6APEAX_KPEAX@ZPEAP6AX11@ZPEAPEAX@Z";

GetBackgroundDrawList :: () -> *ImDrawList #foreign imgui "?GetBackgroundDrawList@ImGui@@YAPEAUImDrawList@@XZ";

GetClipboardText :: () -> *u8 #foreign imgui "?GetClipboardText@ImGui@@YAPEBDXZ";

GetColorU32 :: (idx: Col, alpha_mul: float = 1.0) -> ImU32 #foreign imgui "?GetColorU32@ImGui@@YAIHM@Z";

GetColorU32 :: (col: *ImVec4) -> ImU32 #foreign imgui "?GetColorU32@ImGui@@YAIAEBUImVec4@@@Z";

GetColorU32 :: (col: ImU32) -> ImU32 #foreign imgui "?GetColorU32@ImGui@@YAII@Z";

GetColumnIndex :: () -> s32 #foreign imgui "?GetColumnIndex@ImGui@@YAHXZ";

GetColumnOffset :: (column_index: s32 = -1) -> float #foreign imgui "?GetColumnOffset@ImGui@@YAMH@Z";

GetColumnsCount :: () -> s32 #foreign imgui "?GetColumnsCount@ImGui@@YAHXZ";

GetColumnWidth :: (column_index: s32 = -1) -> float #foreign imgui "?GetColumnWidth@ImGui@@YAMH@Z";

GetContentRegionAvail :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetContentRegionAvail@ImGui@@YA?AUImVec2@@XZ";

GetContentRegionMax :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetContentRegionMax@ImGui@@YA?AUImVec2@@XZ";

GetCurrentContext :: () -> *ImGuiContext #foreign imgui "?GetCurrentContext@ImGui@@YAPEAUImGuiContext@@XZ";

GetCursorPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetCursorPos@ImGui@@YA?AUImVec2@@XZ";

GetCursorPosX :: () -> float #foreign imgui "?GetCursorPosX@ImGui@@YAMXZ";

GetCursorPosY :: () -> float #foreign imgui "?GetCursorPosY@ImGui@@YAMXZ";

GetCursorScreenPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetCursorScreenPos@ImGui@@YA?AUImVec2@@XZ";

GetCursorStartPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetCursorStartPos@ImGui@@YA?AUImVec2@@XZ";

GetDragDropPayload :: () -> *Payload #foreign imgui "?GetDragDropPayload@ImGui@@YAPEBUImGuiPayload@@XZ";

GetDrawData :: () -> *ImDrawData #foreign imgui "?GetDrawData@ImGui@@YAPEAUImDrawData@@XZ";

GetDrawListSharedData :: () -> *ImDrawListSharedData #foreign imgui "?GetDrawListSharedData@ImGui@@YAPEAUImDrawListSharedData@@XZ";

GetFont :: () -> *ImFont #foreign imgui "?GetFont@ImGui@@YAPEAUImFont@@XZ";

GetFontSize :: () -> float #foreign imgui "?GetFontSize@ImGui@@YAMXZ";

GetFontTexUvWhitePixel :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetFontTexUvWhitePixel@ImGui@@YA?AUImVec2@@XZ";

GetForegroundDrawList :: () -> *ImDrawList #foreign imgui "?GetForegroundDrawList@ImGui@@YAPEAUImDrawList@@XZ";

GetFrameCount :: () -> s32 #foreign imgui "?GetFrameCount@ImGui@@YAHXZ";

GetFrameHeight :: () -> float #foreign imgui "?GetFrameHeight@ImGui@@YAMXZ";

GetFrameHeightWithSpacing :: () -> float #foreign imgui "?GetFrameHeightWithSpacing@ImGui@@YAMXZ";

GetID :: (str_id: *u8) -> ID #foreign imgui "?GetID@ImGui@@YAIPEBD@Z";

GetID :: (str_id_begin: *u8, str_id_end: *u8) -> ID #foreign imgui "?GetID@ImGui@@YAIPEBD0@Z";

GetID :: (ptr_id: *void) -> ID #foreign imgui "?GetID@ImGui@@YAIPEBX@Z";

GetIO :: () -> *IO #foreign imgui "?GetIO@ImGui@@YAAEAUImGuiIO@@XZ";

GetItemID :: () -> ID #foreign imgui "?GetItemID@ImGui@@YAIXZ";

GetItemRectMax :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetItemRectMax@ImGui@@YA?AUImVec2@@XZ";

GetItemRectMin :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetItemRectMin@ImGui@@YA?AUImVec2@@XZ";

GetItemRectSize :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetItemRectSize@ImGui@@YA?AUImVec2@@XZ";

GetKeyIndex :: (key: Key) -> Key #foreign imgui "?GetKeyIndex@ImGui@@YA?AW4ImGuiKey@@W42@@Z";

GetKeyName :: (key: Key) -> *u8 #foreign imgui "?GetKeyName@ImGui@@YAPEBDW4ImGuiKey@@@Z";

GetKeyPressedAmount :: (key: Key, repeat_delay: float, rate: float) -> s32 #foreign imgui "?GetKeyPressedAmount@ImGui@@YAHW4ImGuiKey@@MM@Z";

GetMainViewport :: () -> *Viewport #foreign imgui "?GetMainViewport@ImGui@@YAPEAUImGuiViewport@@XZ";

GetMouseClickedCount :: (button: MouseButton) -> s32 #foreign imgui "?GetMouseClickedCount@ImGui@@YAHH@Z";

GetMouseCursor :: () -> MouseCursor #foreign imgui "?GetMouseCursor@ImGui@@YAHXZ";

GetMouseDragDelta :: (button: MouseButton = .Left, lock_threshold: float = -1.0) -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetMouseDragDelta@ImGui@@YA?AUImVec2@@HM@Z";

GetMousePos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetMousePos@ImGui@@YA?AUImVec2@@XZ";

GetMousePosOnOpeningCurrentPopup :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetMousePosOnOpeningCurrentPopup@ImGui@@YA?AUImVec2@@XZ";

GetScrollMaxX :: () -> float #foreign imgui "?GetScrollMaxX@ImGui@@YAMXZ";

GetScrollMaxY :: () -> float #foreign imgui "?GetScrollMaxY@ImGui@@YAMXZ";

GetScrollX :: () -> float #foreign imgui "?GetScrollX@ImGui@@YAMXZ";

GetScrollY :: () -> float #foreign imgui "?GetScrollY@ImGui@@YAMXZ";

GetStateStorage :: () -> *Storage #foreign imgui "?GetStateStorage@ImGui@@YAPEAUImGuiStorage@@XZ";

GetStyle :: () -> *Style #foreign imgui "?GetStyle@ImGui@@YAAEAUImGuiStyle@@XZ";

GetStyleColorName :: (idx: Col) -> *u8 #foreign imgui "?GetStyleColorName@ImGui@@YAPEBDH@Z";

GetStyleColorVec4 :: (idx: Col) -> *ImVec4 #foreign imgui "?GetStyleColorVec4@ImGui@@YAAEBUImVec4@@H@Z";

GetTextLineHeight :: () -> float #foreign imgui "?GetTextLineHeight@ImGui@@YAMXZ";

GetTextLineHeightWithSpacing :: () -> float #foreign imgui "?GetTextLineHeightWithSpacing@ImGui@@YAMXZ";

GetTime :: () -> float64 #foreign imgui "?GetTime@ImGui@@YANXZ";

GetTreeNodeToLabelSpacing :: () -> float #foreign imgui "?GetTreeNodeToLabelSpacing@ImGui@@YAMXZ";

GetVersion :: () -> *u8 #foreign imgui "?GetVersion@ImGui@@YAPEBDXZ";

GetWindowContentRegionMax :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowContentRegionMax@ImGui@@YA?AUImVec2@@XZ";

GetWindowContentRegionMin :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowContentRegionMin@ImGui@@YA?AUImVec2@@XZ";

GetWindowDrawList :: () -> *ImDrawList #foreign imgui "?GetWindowDrawList@ImGui@@YAPEAUImDrawList@@XZ";

GetWindowHeight :: () -> float #foreign imgui "?GetWindowHeight@ImGui@@YAMXZ";

GetWindowPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowPos@ImGui@@YA?AUImVec2@@XZ";

GetWindowSize :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowSize@ImGui@@YA?AUImVec2@@XZ";

GetWindowWidth :: () -> float #foreign imgui "?GetWindowWidth@ImGui@@YAMXZ";

Image :: (user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, tint_col: *ImVec4, border_col: *ImVec4) -> void #foreign imgui "?Image@ImGui@@YAXPEAXAEBUImVec2@@11AEBUImVec4@@2@Z";

ImageButton :: (str_id: *u8, user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, bg_col: *ImVec4, tint_col: *ImVec4) -> bool #foreign imgui "?ImageButton@ImGui@@YA_NPEBDPEAXAEBUImVec2@@22AEBUImVec4@@3@Z";

ImageButton :: (user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, frame_padding: s32 = -1, bg_col: *ImVec4, tint_col: *ImVec4) -> bool #foreign imgui "?ImageButton@ImGui@@YA_NPEAXAEBUImVec2@@11HAEBUImVec4@@2@Z";

Indent :: (indent_w: float = 0.0) -> void #foreign imgui "?Indent@ImGui@@YAXM@Z";

InputDouble :: (label: *u8, v: *float64, step: float64 = 0.0, step_fast: float64 = 0.0, format: *u8 = "%.6f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputDouble@ImGui@@YA_NPEBDPEANNN0H@Z";

InputFloat :: (label: *u8, v: *float, step: float = 0.0, step_fast: float = 0.0, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat@ImGui@@YA_NPEBDPEAMMM0H@Z";

InputFloat2 :: (label: *u8, v: *[2] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat2@ImGui@@YA_NPEBDQEAM0H@Z";

InputFloat3 :: (label: *u8, v: *[3] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat3@ImGui@@YA_NPEBDQEAM0H@Z";

InputFloat4 :: (label: *u8, v: *[4] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat4@ImGui@@YA_NPEBDQEAM0H@Z";

InputInt :: (label: *u8, v: *s32, step: s32 = 1, step_fast: s32 = 100, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt@ImGui@@YA_NPEBDPEAHHHH@Z";

InputInt2 :: (label: *u8, v: *[2] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt2@ImGui@@YA_NPEBDQEAHH@Z";

InputInt3 :: (label: *u8, v: *[3] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt3@ImGui@@YA_NPEBDQEAHH@Z";

InputInt4 :: (label: *u8, v: *[4] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt4@ImGui@@YA_NPEBDQEAHH@Z";

InputScalar :: (label: *u8, data_type: DataType, p_data: *void, p_step: *void = null, p_step_fast: *void = null, format: *u8 = null, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputScalar@ImGui@@YA_NPEBDHPEAXPEBX20H@Z";

InputScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, p_step: *void = null, p_step_fast: *void = null, format: *u8 = null, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputScalarN@ImGui@@YA_NPEBDHPEAXHPEBX20H@Z";

InputText :: (label: *u8, buf: *u8, buf_size: u64, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "?InputText@ImGui@@YA_NPEBDPEAD_KHP6AHPEAUImGuiInputTextCallbackData@@@ZPEAX@Z";

InputTextMultiline :: (label: *u8, buf: *u8, buf_size: u64, size: *ImVec2, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "?InputTextMultiline@ImGui@@YA_NPEBDPEAD_KAEBUImVec2@@HP6AHPEAUImGuiInputTextCallbackData@@@ZPEAX@Z";

InputTextWithHint :: (label: *u8, hint: *u8, buf: *u8, buf_size: u64, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "?InputTextWithHint@ImGui@@YA_NPEBD0PEAD_KHP6AHPEAUImGuiInputTextCallbackData@@@ZPEAX@Z";

InvisibleButton :: (str_id: *u8, size: *ImVec2, flags: ButtonFlags = .None) -> bool #foreign imgui "?InvisibleButton@ImGui@@YA_NPEBDAEBUImVec2@@H@Z";

IsAnyItemActive :: () -> bool #foreign imgui "?IsAnyItemActive@ImGui@@YA_NXZ";

IsAnyItemFocused :: () -> bool #foreign imgui "?IsAnyItemFocused@ImGui@@YA_NXZ";

IsAnyItemHovered :: () -> bool #foreign imgui "?IsAnyItemHovered@ImGui@@YA_NXZ";

IsAnyMouseDown :: () -> bool #foreign imgui "?IsAnyMouseDown@ImGui@@YA_NXZ";

IsItemActivated :: () -> bool #foreign imgui "?IsItemActivated@ImGui@@YA_NXZ";

IsItemActive :: () -> bool #foreign imgui "?IsItemActive@ImGui@@YA_NXZ";

IsItemClicked :: (mouse_button: MouseButton = .Left) -> bool #foreign imgui "?IsItemClicked@ImGui@@YA_NH@Z";

IsItemDeactivated :: () -> bool #foreign imgui "?IsItemDeactivated@ImGui@@YA_NXZ";

IsItemDeactivatedAfterEdit :: () -> bool #foreign imgui "?IsItemDeactivatedAfterEdit@ImGui@@YA_NXZ";

IsItemEdited :: () -> bool #foreign imgui "?IsItemEdited@ImGui@@YA_NXZ";

IsItemFocused :: () -> bool #foreign imgui "?IsItemFocused@ImGui@@YA_NXZ";

IsItemHovered :: (flags: HoveredFlags = .None) -> bool #foreign imgui "?IsItemHovered@ImGui@@YA_NH@Z";

IsItemToggledOpen :: () -> bool #foreign imgui "?IsItemToggledOpen@ImGui@@YA_NXZ";

IsItemVisible :: () -> bool #foreign imgui "?IsItemVisible@ImGui@@YA_NXZ";

IsKeyDown :: (key: Key) -> bool #foreign imgui "?IsKeyDown@ImGui@@YA_NW4ImGuiKey@@@Z";

IsKeyPressed :: (key: Key, repeat := true) -> bool #foreign imgui "?IsKeyPressed@ImGui@@YA_NW4ImGuiKey@@_N@Z";

IsKeyReleased :: (key: Key) -> bool #foreign imgui "?IsKeyReleased@ImGui@@YA_NW4ImGuiKey@@@Z";

IsMouseClicked :: (button: MouseButton, repeat := false) -> bool #foreign imgui "?IsMouseClicked@ImGui@@YA_NH_N@Z";

IsMouseDoubleClicked :: (button: MouseButton) -> bool #foreign imgui "?IsMouseDoubleClicked@ImGui@@YA_NH@Z";

IsMouseDown :: (button: MouseButton) -> bool #foreign imgui "?IsMouseDown@ImGui@@YA_NH@Z";

IsMouseDragging :: (button: MouseButton, lock_threshold: float = -1.0) -> bool #foreign imgui "?IsMouseDragging@ImGui@@YA_NHM@Z";

IsMouseHoveringRect :: (r_min: *ImVec2, r_max: *ImVec2, clip := true) -> bool #foreign imgui "?IsMouseHoveringRect@ImGui@@YA_NAEBUImVec2@@0_N@Z";

IsMousePosValid :: (mouse_pos: *ImVec2 = null) -> bool #foreign imgui "?IsMousePosValid@ImGui@@YA_NPEBUImVec2@@@Z";

IsMouseReleased :: (button: MouseButton) -> bool #foreign imgui "?IsMouseReleased@ImGui@@YA_NH@Z";

IsPopupOpen :: (str_id: *u8, flags: PopupFlags = .None) -> bool #foreign imgui "?IsPopupOpen@ImGui@@YA_NPEBDH@Z";

IsRectVisible :: (size: *ImVec2) -> bool #foreign imgui "?IsRectVisible@ImGui@@YA_NAEBUImVec2@@@Z";

IsRectVisible :: (rect_min: *ImVec2, rect_max: *ImVec2) -> bool #foreign imgui "?IsRectVisible@ImGui@@YA_NAEBUImVec2@@0@Z";

IsWindowAppearing :: () -> bool #foreign imgui "?IsWindowAppearing@ImGui@@YA_NXZ";

IsWindowCollapsed :: () -> bool #foreign imgui "?IsWindowCollapsed@ImGui@@YA_NXZ";

IsWindowFocused :: (flags: FocusedFlags = .None) -> bool #foreign imgui "?IsWindowFocused@ImGui@@YA_NH@Z";

IsWindowHovered :: (flags: HoveredFlags = .None) -> bool #foreign imgui "?IsWindowHovered@ImGui@@YA_NH@Z";

LabelText :: (label: *u8, fmt: *u8, __args: ..Any) -> void #foreign imgui "?LabelText@ImGui@@YAXPEBD0ZZ";

ListBox :: (label: *u8, current_item: *s32, items: **u8, items_count: s32, height_in_items: s32 = -1) -> bool #foreign imgui "?ListBox@ImGui@@YA_NPEBDPEAHQEBQEBDHH@Z";

ListBox :: (label: *u8, current_item: *s32, items_getter: #type (data: *void, idx: s32, out_text: **u8) -> bool #c_call, data: *void, items_count: s32, height_in_items: s32 = -1) -> bool #foreign imgui "?ListBox@ImGui@@YA_NPEBDPEAHP6A_NPEAXHPEAPEBD@Z2HH@Z";

LoadIniSettingsFromDisk :: (ini_filename: *u8) -> void #foreign imgui "?LoadIniSettingsFromDisk@ImGui@@YAXPEBD@Z";

LoadIniSettingsFromMemory :: (ini_data: *u8, ini_size: u64 = 0) -> void #foreign imgui "?LoadIniSettingsFromMemory@ImGui@@YAXPEBD_K@Z";

LogButtons :: () -> void #foreign imgui "?LogButtons@ImGui@@YAXXZ";

LogFinish :: () -> void #foreign imgui "?LogFinish@ImGui@@YAXXZ";

LogText :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?LogText@ImGui@@YAXPEBDZZ";

LogToClipboard :: (auto_open_depth: s32 = -1) -> void #foreign imgui "?LogToClipboard@ImGui@@YAXH@Z";

LogToFile :: (auto_open_depth: s32 = -1, filename: *u8 = null) -> void #foreign imgui "?LogToFile@ImGui@@YAXHPEBD@Z";

LogToTTY :: (auto_open_depth: s32 = -1) -> void #foreign imgui "?LogToTTY@ImGui@@YAXH@Z";

MemAlloc :: (size: u64) -> *void #foreign imgui "?MemAlloc@ImGui@@YAPEAX_K@Z";

MemFree :: (ptr: *void) -> void #foreign imgui "?MemFree@ImGui@@YAXPEAX@Z";

MenuItem :: (label: *u8, shortcut: *u8 = null, selected := false, enabled := true) -> bool #foreign imgui "?MenuItem@ImGui@@YA_NPEBD0_N1@Z";

MenuItem :: (label: *u8, shortcut: *u8, p_selected: *bool, enabled := true) -> bool #foreign imgui "?MenuItem@ImGui@@YA_NPEBD0PEA_N_N@Z";

NewFrame :: () -> void #foreign imgui "?NewFrame@ImGui@@YAXXZ";

NewLine :: () -> void #foreign imgui "?NewLine@ImGui@@YAXXZ";

NextColumn :: () -> void #foreign imgui "?NextColumn@ImGui@@YAXXZ";

OpenPopup :: (str_id: *u8, popup_flags: PopupFlags = .None) -> void #foreign imgui "?OpenPopup@ImGui@@YAXPEBDH@Z";

OpenPopup :: (id: ID, popup_flags: PopupFlags = .None) -> void #foreign imgui "?OpenPopup@ImGui@@YAXIH@Z";

OpenPopupOnItemClick :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> void #foreign imgui "?OpenPopupOnItemClick@ImGui@@YAXPEBDH@Z";

PlotHistogram :: (label: *u8, values: *float, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }, stride: s32 = 4) -> void #foreign imgui "?PlotHistogram@ImGui@@YAXPEBDPEBMHH0MMUImVec2@@H@Z";

PlotHistogram :: (label: *u8, values_getter: #type (data: *void, idx: s32) -> float #c_call, data: *void, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }) -> void #foreign imgui "?PlotHistogram@ImGui@@YAXPEBDP6AMPEAXH@Z1HH0MMUImVec2@@@Z";

PlotLines :: (label: *u8, values: *float, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }, stride: s32 = 4) -> void #foreign imgui "?PlotLines@ImGui@@YAXPEBDPEBMHH0MMUImVec2@@H@Z";

PlotLines :: (label: *u8, values_getter: #type (data: *void, idx: s32) -> float #c_call, data: *void, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }) -> void #foreign imgui "?PlotLines@ImGui@@YAXPEBDP6AMPEAXH@Z1HH0MMUImVec2@@@Z";

PopButtonRepeat :: () -> void #foreign imgui "?PopButtonRepeat@ImGui@@YAXXZ";

PopClipRect :: () -> void #foreign imgui "?PopClipRect@ImGui@@YAXXZ";

PopFont :: () -> void #foreign imgui "?PopFont@ImGui@@YAXXZ";

PopID :: () -> void #foreign imgui "?PopID@ImGui@@YAXXZ";

PopItemWidth :: () -> void #foreign imgui "?PopItemWidth@ImGui@@YAXXZ";

PopStyleColor :: (count: s32 = 1) -> void #foreign imgui "?PopStyleColor@ImGui@@YAXH@Z";

PopStyleVar :: (count: s32 = 1) -> void #foreign imgui "?PopStyleVar@ImGui@@YAXH@Z";

PopTabStop :: () -> void #foreign imgui "?PopTabStop@ImGui@@YAXXZ";

PopTextWrapPos :: () -> void #foreign imgui "?PopTextWrapPos@ImGui@@YAXXZ";

ProgressBar :: (fraction: float, size_arg: *ImVec2, overlay: *u8 = null) -> void #foreign imgui "?ProgressBar@ImGui@@YAXMAEBUImVec2@@PEBD@Z";

PushButtonRepeat :: (repeat: bool) -> void #foreign imgui "?PushButtonRepeat@ImGui@@YAX_N@Z";

PushClipRect :: (clip_rect_min: *ImVec2, clip_rect_max: *ImVec2, intersect_with_current_clip_rect: bool) -> void #foreign imgui "?PushClipRect@ImGui@@YAXAEBUImVec2@@0_N@Z";

PushFont :: (font: *ImFont) -> void #foreign imgui "?PushFont@ImGui@@YAXPEAUImFont@@@Z";

PushID :: (str_id: *u8) -> void #foreign imgui "?PushID@ImGui@@YAXPEBD@Z";

PushID :: (str_id_begin: *u8, str_id_end: *u8) -> void #foreign imgui "?PushID@ImGui@@YAXPEBD0@Z";

PushID :: (ptr_id: *void) -> void #foreign imgui "?PushID@ImGui@@YAXPEBX@Z";

PushID :: (int_id: s32) -> void #foreign imgui "?PushID@ImGui@@YAXH@Z";

PushItemWidth :: (item_width: float) -> void #foreign imgui "?PushItemWidth@ImGui@@YAXM@Z";

PushStyleColor :: (idx: Col, col: ImU32) -> void #foreign imgui "?PushStyleColor@ImGui@@YAXHI@Z";

PushStyleColor :: (idx: Col, col: *ImVec4) -> void #foreign imgui "?PushStyleColor@ImGui@@YAXHAEBUImVec4@@@Z";

PushStyleVar :: (idx: StyleVar, val: float) -> void #foreign imgui "?PushStyleVar@ImGui@@YAXHM@Z";

PushStyleVar :: (idx: StyleVar, val: *ImVec2) -> void #foreign imgui "?PushStyleVar@ImGui@@YAXHAEBUImVec2@@@Z";

PushTabStop :: (tab_stop: bool) -> void #foreign imgui "?PushTabStop@ImGui@@YAX_N@Z";

PushTextWrapPos :: (wrap_local_pos_x: float = 0.0) -> void #foreign imgui "?PushTextWrapPos@ImGui@@YAXM@Z";

RadioButton :: (label: *u8, active: bool) -> bool #foreign imgui "?RadioButton@ImGui@@YA_NPEBD_N@Z";

RadioButton :: (label: *u8, v: *s32, v_button: s32) -> bool #foreign imgui "?RadioButton@ImGui@@YA_NPEBDPEAHH@Z";

Render :: () -> void #foreign imgui "?Render@ImGui@@YAXXZ";

ResetMouseDragDelta :: (button: MouseButton = .Left) -> void #foreign imgui "?ResetMouseDragDelta@ImGui@@YAXH@Z";

SameLine :: (offset_from_start_x: float = 0.0, spacing: float = -1.0) -> void #foreign imgui "?SameLine@ImGui@@YAXMM@Z";

SaveIniSettingsToDisk :: (ini_filename: *u8) -> void #foreign imgui "?SaveIniSettingsToDisk@ImGui@@YAXPEBD@Z";

SaveIniSettingsToMemory :: (out_ini_size: *u64 = null) -> *u8 #foreign imgui "?SaveIniSettingsToMemory@ImGui@@YAPEBDPEA_K@Z";

Selectable :: (label: *u8, selected := false, flags: SelectableFlags = .None, size: *ImVec2) -> bool #foreign imgui "?Selectable@ImGui@@YA_NPEBD_NHAEBUImVec2@@@Z";

Selectable :: (label: *u8, p_selected: *bool, flags: SelectableFlags = .None, size: *ImVec2) -> bool #foreign imgui "?Selectable@ImGui@@YA_NPEBDPEA_NHAEBUImVec2@@@Z";

Separator :: () -> void #foreign imgui "?Separator@ImGui@@YAXXZ";

SeparatorText :: (label: *u8) -> void #foreign imgui "?SeparatorText@ImGui@@YAXPEBD@Z";

SetAllocatorFunctions :: (alloc_func: MemAllocFunc, free_func: MemFreeFunc, user_data: *void = null) -> void #foreign imgui "?SetAllocatorFunctions@ImGui@@YAXP6APEAX_KPEAX@ZP6AX11@Z1@Z";

SetClipboardText :: (text: *u8) -> void #foreign imgui "?SetClipboardText@ImGui@@YAXPEBD@Z";

SetColorEditOptions :: (flags: ColorEditFlags) -> void #foreign imgui "?SetColorEditOptions@ImGui@@YAXH@Z";

SetColumnOffset :: (column_index: s32, offset_x: float) -> void #foreign imgui "?SetColumnOffset@ImGui@@YAXHM@Z";

SetColumnWidth :: (column_index: s32, width: float) -> void #foreign imgui "?SetColumnWidth@ImGui@@YAXHM@Z";

SetCurrentContext :: (ctx: *ImGuiContext) -> void #foreign imgui "?SetCurrentContext@ImGui@@YAXPEAUImGuiContext@@@Z";

SetCursorPos :: (local_pos: *ImVec2) -> void #foreign imgui "?SetCursorPos@ImGui@@YAXAEBUImVec2@@@Z";

SetCursorPosX :: (local_x: float) -> void #foreign imgui "?SetCursorPosX@ImGui@@YAXM@Z";

SetCursorPosY :: (local_y: float) -> void #foreign imgui "?SetCursorPosY@ImGui@@YAXM@Z";

SetCursorScreenPos :: (pos: *ImVec2) -> void #foreign imgui "?SetCursorScreenPos@ImGui@@YAXAEBUImVec2@@@Z";

SetDragDropPayload :: (type: *u8, data: *void, sz: u64, cond: Cond = .None) -> bool #foreign imgui "?SetDragDropPayload@ImGui@@YA_NPEBDPEBX_KH@Z";

SetItemAllowOverlap :: () -> void #foreign imgui "?SetItemAllowOverlap@ImGui@@YAXXZ";

SetItemDefaultFocus :: () -> void #foreign imgui "?SetItemDefaultFocus@ImGui@@YAXXZ";

SetKeyboardFocusHere :: (offset: s32 = 0) -> void #foreign imgui "?SetKeyboardFocusHere@ImGui@@YAXH@Z";

SetMouseCursor :: (cursor_type: MouseCursor) -> void #foreign imgui "?SetMouseCursor@ImGui@@YAXH@Z";

SetNextFrameWantCaptureKeyboard :: (want_capture_keyboard: bool) -> void #foreign imgui "?SetNextFrameWantCaptureKeyboard@ImGui@@YAX_N@Z";

SetNextFrameWantCaptureMouse :: (want_capture_mouse: bool) -> void #foreign imgui "?SetNextFrameWantCaptureMouse@ImGui@@YAX_N@Z";

SetNextItemOpen :: (is_open: bool, cond: Cond = .None) -> void #foreign imgui "?SetNextItemOpen@ImGui@@YAX_NH@Z";

SetNextItemWidth :: (item_width: float) -> void #foreign imgui "?SetNextItemWidth@ImGui@@YAXM@Z";

SetNextWindowBgAlpha :: (alpha: float) -> void #foreign imgui "?SetNextWindowBgAlpha@ImGui@@YAXM@Z";

SetNextWindowCollapsed :: (collapsed: bool, cond: Cond = .None) -> void #foreign imgui "?SetNextWindowCollapsed@ImGui@@YAX_NH@Z";

SetNextWindowContentSize :: (size: *ImVec2) -> void #foreign imgui "?SetNextWindowContentSize@ImGui@@YAXAEBUImVec2@@@Z";

SetNextWindowFocus :: () -> void #foreign imgui "?SetNextWindowFocus@ImGui@@YAXXZ";

SetNextWindowPos :: (pos: *ImVec2, cond: Cond = .None, pivot: *ImVec2) -> void #foreign imgui "?SetNextWindowPos@ImGui@@YAXAEBUImVec2@@H0@Z";

SetNextWindowScroll :: (scroll: *ImVec2) -> void #foreign imgui "?SetNextWindowScroll@ImGui@@YAXAEBUImVec2@@@Z";

SetNextWindowSize :: (size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetNextWindowSize@ImGui@@YAXAEBUImVec2@@H@Z";

SetNextWindowSizeConstraints :: (size_min: *ImVec2, size_max: *ImVec2, custom_callback: SizeCallback = null, custom_callback_data: *void = null) -> void #foreign imgui "?SetNextWindowSizeConstraints@ImGui@@YAXAEBUImVec2@@0P6AXPEAUImGuiSizeCallbackData@@@ZPEAX@Z";

SetScrollFromPosX :: (local_x: float, center_x_ratio: float = 0.5) -> void #foreign imgui "?SetScrollFromPosX@ImGui@@YAXMM@Z";

SetScrollFromPosY :: (local_y: float, center_y_ratio: float = 0.5) -> void #foreign imgui "?SetScrollFromPosY@ImGui@@YAXMM@Z";

SetScrollHereX :: (center_x_ratio: float = 0.5) -> void #foreign imgui "?SetScrollHereX@ImGui@@YAXM@Z";

SetScrollHereY :: (center_y_ratio: float = 0.5) -> void #foreign imgui "?SetScrollHereY@ImGui@@YAXM@Z";

SetScrollX :: (scroll_x: float) -> void #foreign imgui "?SetScrollX@ImGui@@YAXM@Z";

SetScrollY :: (scroll_y: float) -> void #foreign imgui "?SetScrollY@ImGui@@YAXM@Z";

SetStateStorage :: (storage: *Storage) -> void #foreign imgui "?SetStateStorage@ImGui@@YAXPEAUImGuiStorage@@@Z";

SetTabItemClosed :: (tab_or_docked_window_label: *u8) -> void #foreign imgui "?SetTabItemClosed@ImGui@@YAXPEBD@Z";

SetTooltip :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?SetTooltip@ImGui@@YAXPEBDZZ";

SetWindowCollapsed :: (collapsed: bool, cond: Cond = .None) -> void #foreign imgui "?SetWindowCollapsed@ImGui@@YAX_NH@Z";

SetWindowCollapsed :: (name: *u8, collapsed: bool, cond: Cond = .None) -> void #foreign imgui "?SetWindowCollapsed@ImGui@@YAXPEBD_NH@Z";

SetWindowFocus :: () -> void #foreign imgui "?SetWindowFocus@ImGui@@YAXXZ";

SetWindowFocus :: (name: *u8) -> void #foreign imgui "?SetWindowFocus@ImGui@@YAXPEBD@Z";

SetWindowFontScale :: (scale: float) -> void #foreign imgui "?SetWindowFontScale@ImGui@@YAXM@Z";

SetWindowPos :: (pos: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowPos@ImGui@@YAXAEBUImVec2@@H@Z";

SetWindowPos :: (name: *u8, pos: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowPos@ImGui@@YAXPEBDAEBUImVec2@@H@Z";

SetWindowSize :: (size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowSize@ImGui@@YAXAEBUImVec2@@H@Z";

SetWindowSize :: (name: *u8, size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowSize@ImGui@@YAXPEBDAEBUImVec2@@H@Z";

ShowAboutWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowAboutWindow@ImGui@@YAXPEA_N@Z";

ShowDebugLogWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowDebugLogWindow@ImGui@@YAXPEA_N@Z";

ShowDemoWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowDemoWindow@ImGui@@YAXPEA_N@Z";

ShowFontSelector :: (label: *u8) -> void #foreign imgui "?ShowFontSelector@ImGui@@YAXPEBD@Z";

ShowMetricsWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowMetricsWindow@ImGui@@YAXPEA_N@Z";

ShowStackToolWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowStackToolWindow@ImGui@@YAXPEA_N@Z";

ShowStyleEditor :: (ref: *Style = null) -> void #foreign imgui "?ShowStyleEditor@ImGui@@YAXPEAUImGuiStyle@@@Z";

ShowStyleSelector :: (label: *u8) -> bool #foreign imgui "?ShowStyleSelector@ImGui@@YA_NPEBD@Z";

ShowUserGuide :: () -> void #foreign imgui "?ShowUserGuide@ImGui@@YAXXZ";

SliderAngle :: (label: *u8, v_rad: *float, v_degrees_min: float = -360.0, v_degrees_max: float = +360.0, format: *u8 = "%.0f deg", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderAngle@ImGui@@YA_NPEBDPEAMMM0H@Z";

SliderFloat :: (label: *u8, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat@ImGui@@YA_NPEBDPEAMMM0H@Z";

SliderFloat2 :: (label: *u8, v: *[2] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat2@ImGui@@YA_NPEBDQEAMMM0H@Z";

SliderFloat3 :: (label: *u8, v: *[3] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat3@ImGui@@YA_NPEBDQEAMMM0H@Z";

SliderFloat4 :: (label: *u8, v: *[4] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat4@ImGui@@YA_NPEBDQEAMMM0H@Z";

SliderInt :: (label: *u8, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt@ImGui@@YA_NPEBDPEAHHH0H@Z";

SliderInt2 :: (label: *u8, v: *[2] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt2@ImGui@@YA_NPEBDQEAHHH0H@Z";

SliderInt3 :: (label: *u8, v: *[3] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt3@ImGui@@YA_NPEBDQEAHHH0H@Z";

SliderInt4 :: (label: *u8, v: *[4] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt4@ImGui@@YA_NPEBDQEAHHH0H@Z";

SliderScalar :: (label: *u8, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?SliderScalar@ImGui@@YA_NPEBDHPEAXPEBX20H@Z";

SliderScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?SliderScalarN@ImGui@@YA_NPEBDHPEAXHPEBX20H@Z";

SmallButton :: (label: *u8) -> bool #foreign imgui "?SmallButton@ImGui@@YA_NPEBD@Z";

Spacing :: () -> void #foreign imgui "?Spacing@ImGui@@YAXXZ";

StyleColorsClassic :: (dst: *Style = null) -> void #foreign imgui "?StyleColorsClassic@ImGui@@YAXPEAUImGuiStyle@@@Z";

StyleColorsDark :: (dst: *Style = null) -> void #foreign imgui "?StyleColorsDark@ImGui@@YAXPEAUImGuiStyle@@@Z";

StyleColorsLight :: (dst: *Style = null) -> void #foreign imgui "?StyleColorsLight@ImGui@@YAXPEAUImGuiStyle@@@Z";

TabItemButton :: (label: *u8, flags: TabItemFlags = .None) -> bool #foreign imgui "?TabItemButton@ImGui@@YA_NPEBDH@Z";

TableGetColumnCount :: () -> s32 #foreign imgui "?TableGetColumnCount@ImGui@@YAHXZ";

TableGetColumnFlags :: (column_n: s32 = -1) -> TableColumnFlags #foreign imgui "?TableGetColumnFlags@ImGui@@YAHH@Z";

TableGetColumnIndex :: () -> s32 #foreign imgui "?TableGetColumnIndex@ImGui@@YAHXZ";

TableGetColumnName :: (column_n: s32 = -1) -> *u8 #foreign imgui "?TableGetColumnName@ImGui@@YAPEBDH@Z";

TableGetRowIndex :: () -> s32 #foreign imgui "?TableGetRowIndex@ImGui@@YAHXZ";

TableGetSortSpecs :: () -> *TableSortSpecs #foreign imgui "?TableGetSortSpecs@ImGui@@YAPEAUImGuiTableSortSpecs@@XZ";

TableHeader :: (label: *u8) -> void #foreign imgui "?TableHeader@ImGui@@YAXPEBD@Z";

TableHeadersRow :: () -> void #foreign imgui "?TableHeadersRow@ImGui@@YAXXZ";

TableNextColumn :: () -> bool #foreign imgui "?TableNextColumn@ImGui@@YA_NXZ";

TableNextRow :: (row_flags: TableRowFlags = .None, min_row_height: float = 0.0) -> void #foreign imgui "?TableNextRow@ImGui@@YAXHM@Z";

TableSetBgColor :: (target: TableBgTarget, color: ImU32, column_n: s32 = -1) -> void #foreign imgui "?TableSetBgColor@ImGui@@YAXHIH@Z";

TableSetColumnEnabled :: (column_n: s32, v: bool) -> void #foreign imgui "?TableSetColumnEnabled@ImGui@@YAXH_N@Z";

TableSetColumnIndex :: (column_n: s32) -> bool #foreign imgui "?TableSetColumnIndex@ImGui@@YA_NH@Z";

TableSetupColumn :: (label: *u8, flags: TableColumnFlags = .None, init_width_or_weight: float = 0.0, user_id: ID = 0) -> void #foreign imgui "?TableSetupColumn@ImGui@@YAXPEBDHMI@Z";

TableSetupScrollFreeze :: (cols: s32, rows: s32) -> void #foreign imgui "?TableSetupScrollFreeze@ImGui@@YAXHH@Z";

Text :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?Text@ImGui@@YAXPEBDZZ";

TextColored :: (col: *ImVec4, fmt: *u8, __args: ..Any) -> void #foreign imgui "?TextColored@ImGui@@YAXAEBUImVec4@@PEBDZZ";

TextDisabled :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?TextDisabled@ImGui@@YAXPEBDZZ";

TextUnformatted :: (text: *u8, text_end: *u8 = null) -> void #foreign imgui "?TextUnformatted@ImGui@@YAXPEBD0@Z";

TextWrapped :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?TextWrapped@ImGui@@YAXPEBDZZ";

TreeNode :: (label: *u8) -> bool #foreign imgui "?TreeNode@ImGui@@YA_NPEBD@Z";

TreeNode :: (str_id: *u8, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNode@ImGui@@YA_NPEBD0ZZ";

TreeNode :: (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNode@ImGui@@YA_NPEBXPEBDZZ";

TreeNodeEx :: (label: *u8, flags: TreeNodeFlags = .None) -> bool #foreign imgui "?TreeNodeEx@ImGui@@YA_NPEBDH@Z";

TreeNodeEx :: (str_id: *u8, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNodeEx@ImGui@@YA_NPEBDH0ZZ";

TreeNodeEx :: (ptr_id: *void, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNodeEx@ImGui@@YA_NPEBXHPEBDZZ";

TreePop :: () -> void #foreign imgui "?TreePop@ImGui@@YAXXZ";

TreePush :: (str_id: *u8) -> void #foreign imgui "?TreePush@ImGui@@YAXPEBD@Z";

TreePush :: (ptr_id: *void) -> void #foreign imgui "?TreePush@ImGui@@YAXPEBX@Z";

Unindent :: (indent_w: float = 0.0) -> void #foreign imgui "?Unindent@ImGui@@YAXM@Z";

Value :: (prefix: *u8, b: bool) -> void #foreign imgui "?Value@ImGui@@YAXPEBD_N@Z";

Value :: (prefix: *u8, v: s32) -> void #foreign imgui "?Value@ImGui@@YAXPEBDH@Z";

Value :: (prefix: *u8, v: u32) -> void #foreign imgui "?Value@ImGui@@YAXPEBDI@Z";

Value :: (prefix: *u8, v: float, float_format: *u8 = null) -> void #foreign imgui "?Value@ImGui@@YAXPEBDM0@Z";

VSliderFloat :: (label: *u8, size: *ImVec2, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?VSliderFloat@ImGui@@YA_NPEBDAEBUImVec2@@PEAMMM0H@Z";

VSliderInt :: (label: *u8, size: *ImVec2, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?VSliderInt@ImGui@@YA_NPEBDAEBUImVec2@@PEAHHH0H@Z";

VSliderScalar :: (label: *u8, size: *ImVec2, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?VSliderScalar@ImGui@@YA_NPEBDAEBUImVec2@@HPEAXPEBX30H@Z";

BeginChild :: (str_id: *u8, size: ImVec2 = ImVec2.{ 0, 0 }, border := false, flags: WindowFlags = .None) -> bool #no_context {
    return BeginChild(str_id, *size, border, flags);
}

BeginChild :: (id: ID, size: ImVec2 = ImVec2.{ 0, 0 }, border := false, flags: WindowFlags = .None) -> bool #no_context {
    return BeginChild(id, *size, border, flags);
}

BeginChildFrame :: (id: ID, size: ImVec2, flags: WindowFlags = .None) -> bool #no_context {
    return BeginChildFrame(id, *size, flags);
}

BeginListBox :: (label: *u8, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
    return BeginListBox(label, *size);
}

BeginTable :: (str_id: *u8, column: s32, flags: TableFlags = .None, outer_size: ImVec2 = ImVec2.{ 0.0, 0.0 }, inner_width: float = 0.0) -> bool #no_context {
    return BeginTable(str_id, column, flags, *outer_size, inner_width);
}

Button :: (label: *u8, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
    return Button(label, *size);
}

CalcTextSize :: (text: string, hide_text_after_double_hash := false, wrap_width: float = -1.0) -> ImVec2 #no_context {
    return CalcTextSize(text.data, text.data + text.count, hide_text_after_double_hash, wrap_width);
}

ColorButton :: (desc_id: *u8, col: ImVec4, flags: ColorEditFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
    return ColorButton(desc_id, *col, flags, *size);
}

ColorConvertFloat4ToU32 :: (in: ImVec4) -> ImU32 #no_context {
    return ColorConvertFloat4ToU32(*in);
}

Dummy :: (size: ImVec2) #no_context {
    Dummy(*size);
}

get_Codepoint :: (fields: ImFontGlyph) -> u32 {
    return cast(u32) ((fields.__bitfield >> 2) & 1073741823);
}

get_Colored :: (fields: ImFontGlyph) -> bool {
    return ((fields.__bitfield >> 0) & 1) != 0;
}

get_Visible :: (fields: ImFontGlyph) -> bool {
    return ((fields.__bitfield >> 1) & 1) != 0;
}

GetColorU32 :: (col: ImVec4) -> ImU32 #no_context {
    return GetColorU32(*col);
}

GetID :: (str_id: string) -> ID #no_context {
    return GetID(str_id.data, str_id.data + str_id.count);
}

Image :: (user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }, border_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }) #no_context {
    Image(user_texture_id, *size, *uv0, *uv1, *tint_col, *border_col);
}

ImageButton :: (str_id: *u8, user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, bg_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }) -> bool #no_context {
    return ImageButton(str_id, user_texture_id, *size, *uv0, *uv1, *bg_col, *tint_col);
}

ImageButton :: (user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, frame_padding: s32 = -1, bg_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }) -> bool #no_context {
    return ImageButton(user_texture_id, *size, *uv0, *uv1, frame_padding, *bg_col, *tint_col);
}

InputTextMultiline :: (label: *u8, buf: *u8, buf_size: u64, size: ImVec2 = ImVec2.{ 0, 0 }, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #no_context {
    return InputTextMultiline(label, buf, buf_size, *size, flags, callback, user_data);
}

InvisibleButton :: (str_id: *u8, size: ImVec2, flags: ButtonFlags = .None) -> bool #no_context {
    return InvisibleButton(str_id, *size, flags);
}

IsMouseHoveringRect :: (r_min: ImVec2, r_max: ImVec2, clip := true) -> bool #no_context {
    return IsMouseHoveringRect(*r_min, *r_max, clip);
}

IsRectVisible :: (size: ImVec2) -> bool #no_context {
    return IsRectVisible(*size);
}

IsRectVisible :: (rect_min: ImVec2, rect_max: ImVec2) -> bool #no_context {
    return IsRectVisible(*rect_min, *rect_max);
}

ProgressBar :: (fraction: float, size_arg: ImVec2 = ImVec2.{ -FLOAT32_MIN, 0 }, overlay: *u8 = null) #no_context {
    ProgressBar(fraction, *size_arg, overlay);
}

PushClipRect :: (clip_rect_min: ImVec2, clip_rect_max: ImVec2, intersect_with_current_clip_rect: bool) #no_context {
    PushClipRect(*clip_rect_min, *clip_rect_max, intersect_with_current_clip_rect);
}

PushID :: (str_id: string) #no_context {
    PushID(str_id.data, str_id.data + str_id.count);
}

PushStyleColor :: (idx: Col, col: ImVec4) #no_context {
    PushStyleColor(idx, *col);
}

PushStyleVar :: (idx: StyleVar, val: ImVec2) #no_context {
    PushStyleVar(idx, *val);
}

Selectable :: (label: *u8, selected := false, flags: SelectableFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
    return Selectable(label, selected, flags, *size);
}

Selectable :: (label: *u8, p_selected: *bool, flags: SelectableFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
    return Selectable(label, p_selected, flags, *size);
}

set_Codepoint :: (fields: *ImFontGlyph, value: u32) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4294967292)) | ((value & 1073741823) << 2);
}

set_Colored :: (fields: *ImFontGlyph, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u32) 1)) | (((cast(u32) value) & 1) << 0);
}

set_Visible :: (fields: *ImFontGlyph, value: bool) {
    fields.__bitfield = (fields.__bitfield & ~(cast(u32) 2)) | (((cast(u32) value) & 1) << 1);
}

SetCursorPos :: (local_pos: ImVec2) #no_context {
    SetCursorPos(*local_pos);
}

SetCursorScreenPos :: (pos: ImVec2) #no_context {
    SetCursorScreenPos(*pos);
}

SetNextWindowContentSize :: (size: ImVec2) #no_context {
    SetNextWindowContentSize(*size);
}

SetNextWindowPos :: (pos: ImVec2, cond: Cond = .None, pivot: ImVec2 = ImVec2.{ 0, 0 }) #no_context {
    SetNextWindowPos(*pos, cond, *pivot);
}

SetNextWindowScroll :: (scroll: ImVec2) #no_context {
    SetNextWindowScroll(*scroll);
}

SetNextWindowSize :: (size: ImVec2, cond: Cond = .None) #no_context {
    SetNextWindowSize(*size, cond);
}

SetNextWindowSizeConstraints :: (size_min: ImVec2, size_max: ImVec2, custom_callback: SizeCallback = null, custom_callback_data: *void = null) #no_context {
    SetNextWindowSizeConstraints(*size_min, *size_max, custom_callback, custom_callback_data);
}

SetWindowPos :: (pos: ImVec2, cond: Cond = .None) #no_context {
    SetWindowPos(*pos, cond);
}

SetWindowPos :: (name: *u8, pos: ImVec2, cond: Cond = .None) #no_context {
    SetWindowPos(name, *pos, cond);
}

SetWindowSize :: (size: ImVec2, cond: Cond = .None) #no_context {
    SetWindowSize(*size, cond);
}

SetWindowSize :: (name: *u8, size: ImVec2, cond: Cond = .None) #no_context {
    SetWindowSize(name, *size, cond);
}

TextUnformatted :: (text: string) #no_context {
    TextUnformatted(text.data, text.data + text.count);
}

VSliderFloat :: (label: *u8, size: ImVec2, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #no_context {
    return VSliderFloat(label, *size, v, v_min, v_max, format, flags);
}

VSliderInt :: (label: *u8, size: ImVec2, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #no_context {
    return VSliderInt(label, *size, v, v_min, v_max, format, flags);
}

VSliderScalar :: (label: *u8, size: ImVec2, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #no_context {
    return VSliderScalar(label, *size, data_type, p_data, p_min, p_max, format, flags);
}
