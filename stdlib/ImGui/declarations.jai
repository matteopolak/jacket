// Generated from docs/specs/stdlib/modules/imgui.md.
// Regenerate with tools/native_bindings/generate.py; do not hand-edit.

#scope_file
#import "Math";
Native :: #import "internal/bindings/native";
Varargs :: #import "internal/bindings/varargs";

#if OS == .MACOS {
    #library,system,link_always "c++";
} else #if OS == .LINUX {
    #library,system,link_always "stdc++";
}

imgui :: #library,system "imgui";
#scope_export

// Types and layouts

#if OS != .WINDOWS {
    BackendFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ButtonFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    Col :: enum u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ColorEditFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ComboFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    Cond :: enum u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ConfigFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    DataType :: enum u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    DragDropFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    FocusedFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    HoveredFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImDrawChannel :: struct {
        _CmdBuffer: ImVector(ImDrawCmd);
        _IdxBuffer: ImVector(ImDrawIdx);
    }

}

#if OS == .WINDOWS {
    ImDrawChannel :: struct {
        _CmdBuffer: ImVector(ImDrawCmd);
        _IdxBuffer: ImVector(ImDrawIdx);
    }

}

#if OS != .WINDOWS {
    ImDrawCmd :: struct {
        ClipRect: ImVec4;
        TextureId: ImTextureID;
        VtxOffset: u32;
        IdxOffset: u32;
        ElemCount: u32;
        UserCallback: ImDrawCallback;
        UserCallbackData: *void;
    }

}

#if OS == .WINDOWS {
    ImDrawCmd :: struct {
        ClipRect: ImVec4;
        TextureId: ImTextureID;
        VtxOffset: u32;
        IdxOffset: u32;
        ElemCount: u32;
        UserCallback: ImDrawCallback;
        UserCallbackData: *void;
    }

}

#if OS != .WINDOWS {
    ImDrawCmdHeader :: struct {
        ClipRect: ImVec4;
        TextureId: ImTextureID;
        VtxOffset: u32;
    }

}

#if OS == .WINDOWS {
    ImDrawCmdHeader :: struct {
        ClipRect: ImVec4;
        TextureId: ImTextureID;
        VtxOffset: u32;
    }

}

#if OS != .WINDOWS {
    ImDrawCornerFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImDrawData :: struct {
        Valid: bool;
        CmdListsCount: s32;
        TotalIdxCount: s32;
        TotalVtxCount: s32;
        CmdLists: **ImDrawList;
        DisplayPos: ImVec2;
        DisplaySize: ImVec2;
        FramebufferScale: ImVec2;
        DeIndexAllBuffers :: (this: *ImDrawData) -> void #cpp_method #foreign imgui "_ZN10ImDrawData17DeIndexAllBuffersEv";
        ScaleClipRects :: (this: *ImDrawData, fb_scale: *ImVec2) -> void #cpp_method #foreign imgui "_ZN10ImDrawData14ScaleClipRectsERK6ImVec2";
        ScaleClipRects :: (this: *ImDrawData, fb_scale: ImVec2) #no_context {
            ScaleClipRects(this, *fb_scale);
        }
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImDrawFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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
        PushClipRect :: (this: *ImDrawList, clip_rect_min: *ImVec2, clip_rect_max: *ImVec2, intersect_with_current_clip_rect := false) -> void #cpp_method #foreign imgui "_ZN10ImDrawList12PushClipRectERK6ImVec2S2_b";
        PushClipRect :: (this: *ImDrawList, clip_rect_min: ImVec2, clip_rect_max: ImVec2, intersect_with_current_clip_rect := false) #no_context {
            PushClipRect(this, *clip_rect_min, *clip_rect_max, intersect_with_current_clip_rect);
        }
        PushClipRectFullScreen :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList22PushClipRectFullScreenEv";
        PopClipRect :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList11PopClipRectEv";
        PushTextureID :: (this: *ImDrawList, texture_id: ImTextureID) -> void #cpp_method #foreign imgui "_ZN10ImDrawList13PushTextureIDEPv";
        PopTextureID :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList12PopTextureIDEv";
        AddLine :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, col: ImU32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList7AddLineERK6ImVec2S2_jf";
        AddLine :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, col: ImU32, thickness: float = 1.0) #no_context {
            AddLine(this, *p1, *p2, col, thickness);
        }
        AddRect :: (this: *ImDrawList, p_min: *ImVec2, p_max: *ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None, thickness: float = 1.0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList7AddRectERK6ImVec2S2_jfif";
        AddRect :: (this: *ImDrawList, p_min: ImVec2, p_max: ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None, thickness: float = 1.0) #no_context {
            AddRect(this, *p_min, *p_max, col, rounding, flags, thickness);
        }
        AddRectFilled :: (this: *ImDrawList, p_min: *ImVec2, p_max: *ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None) -> void #cpp_method #foreign imgui "_ZN10ImDrawList13AddRectFilledERK6ImVec2S2_jfi";
        AddRectFilled :: (this: *ImDrawList, p_min: ImVec2, p_max: ImVec2, col: ImU32, rounding: float = 0.0, flags: ImDrawFlags = .None) #no_context {
            AddRectFilled(this, *p_min, *p_max, col, rounding, flags);
        }
        AddRectFilledMultiColor :: (this: *ImDrawList, p_min: *ImVec2, p_max: *ImVec2, col_upr_left: ImU32, col_upr_right: ImU32, col_bot_right: ImU32, col_bot_left: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList23AddRectFilledMultiColorERK6ImVec2S2_jjjj";
        AddRectFilledMultiColor :: (this: *ImDrawList, p_min: ImVec2, p_max: ImVec2, col_upr_left: ImU32, col_upr_right: ImU32, col_bot_right: ImU32, col_bot_left: ImU32) #no_context {
            AddRectFilledMultiColor(this, *p_min, *p_max, col_upr_left, col_upr_right, col_bot_right, col_bot_left);
        }
        AddQuad :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, col: ImU32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList7AddQuadERK6ImVec2S2_S2_S2_jf";
        AddQuad :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, col: ImU32, thickness: float = 1.0) #no_context {
            AddQuad(this, *p1, *p2, *p3, *p4, col, thickness);
        }
        AddQuadFilled :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList13AddQuadFilledERK6ImVec2S2_S2_S2_j";
        AddQuadFilled :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, col: ImU32) #no_context {
            AddQuadFilled(this, *p1, *p2, *p3, *p4, col);
        }
        AddTriangle :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, col: ImU32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList11AddTriangleERK6ImVec2S2_S2_jf";
        AddTriangle :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, col: ImU32, thickness: float = 1.0) #no_context {
            AddTriangle(this, *p1, *p2, *p3, col, thickness);
        }
        AddTriangleFilled :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList17AddTriangleFilledERK6ImVec2S2_S2_j";
        AddTriangleFilled :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, col: ImU32) #no_context {
            AddTriangleFilled(this, *p1, *p2, *p3, col);
        }
        AddCircle :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32 = 0, thickness: float = 1.0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList9AddCircleERK6ImVec2fjif";
        AddCircle :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32 = 0, thickness: float = 1.0) #no_context {
            AddCircle(this, *center, radius, col, num_segments, thickness);
        }
        AddCircleFilled :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList15AddCircleFilledERK6ImVec2fji";
        AddCircleFilled :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32 = 0) #no_context {
            AddCircleFilled(this, *center, radius, col, num_segments);
        }
        AddNgon :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32, thickness: float = 1.0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList7AddNgonERK6ImVec2fjif";
        AddNgon :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32, thickness: float = 1.0) #no_context {
            AddNgon(this, *center, radius, col, num_segments, thickness);
        }
        AddNgonFilled :: (this: *ImDrawList, center: *ImVec2, radius: float, col: ImU32, num_segments: s32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList13AddNgonFilledERK6ImVec2fji";
        AddNgonFilled :: (this: *ImDrawList, center: ImVec2, radius: float, col: ImU32, num_segments: s32) #no_context {
            AddNgonFilled(this, *center, radius, col, num_segments);
        }
        AddText :: (this: *ImDrawList, pos: *ImVec2, col: ImU32, text_begin: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "_ZN10ImDrawList7AddTextERK6ImVec2jPKcS4_";
        AddText :: (this: *ImDrawList, pos: ImVec2, col: ImU32, text: string) #no_context {
            AddText(this, *pos, col, text.data, text.data + text.count);
        }
        AddText :: (this: *ImDrawList, font: *ImFont, font_size: float, pos: *ImVec2, col: ImU32, text_begin: *u8, text_end: *u8 = null, wrap_width: float = 0.0, cpu_fine_clip_rect: *ImVec4 = null) -> void #cpp_method #foreign imgui "_ZN10ImDrawList7AddTextEPK6ImFontfRK6ImVec2jPKcS7_fPK6ImVec4";
        AddText :: (this: *ImDrawList, font: *ImFont, font_size: float, pos: ImVec2, col: ImU32, text: string, wrap_width: float = 0.0, cpu_fine_clip_rect: *ImVec4 = null) #no_context {
            AddText(this, font, font_size, *pos, col, text.data, text.data + text.count, wrap_width, cpu_fine_clip_rect);
        }
        AddPolyline :: (this: *ImDrawList, points: *ImVec2, num_points: s32, col: ImU32, flags: ImDrawFlags, thickness: float) -> void #cpp_method #foreign imgui "_ZN10ImDrawList11AddPolylineEPK6ImVec2ijif";
        AddConvexPolyFilled :: (this: *ImDrawList, points: *ImVec2, num_points: s32, col: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList19AddConvexPolyFilledEPK6ImVec2ij";
        AddBezierCubic :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList14AddBezierCubicERK6ImVec2S2_S2_S2_jfi";
        AddBezierCubic :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) #no_context {
            AddBezierCubic(this, *p1, *p2, *p3, *p4, col, thickness, num_segments);
        }
        AddBezierQuadratic :: (this: *ImDrawList, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList18AddBezierQuadraticERK6ImVec2S2_S2_jfi";
        AddBezierQuadratic :: (this: *ImDrawList, p1: ImVec2, p2: ImVec2, p3: ImVec2, col: ImU32, thickness: float, num_segments: s32 = 0) #no_context {
            AddBezierQuadratic(this, *p1, *p2, *p3, col, thickness, num_segments);
        }
        AddImage :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: *ImVec2, p_max: *ImVec2, uv_min: *ImVec2, uv_max: *ImVec2, col: ImU32 = 4294967295) -> void #cpp_method #foreign imgui "_ZN10ImDrawList8AddImageEPvRK6ImVec2S3_S3_S3_j";
        AddImage :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: ImVec2, p_max: ImVec2, uv_min: ImVec2 = ImVec2.{ 0, 0 }, uv_max: ImVec2 = ImVec2.{ 1, 1 }, col: ImU32 = 4294967295) #no_context {
            AddImage(this, user_texture_id, *p_min, *p_max, *uv_min, *uv_max, col);
        }
        AddImageQuad :: (this: *ImDrawList, user_texture_id: ImTextureID, p1: *ImVec2, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, uv1: *ImVec2, uv2: *ImVec2, uv3: *ImVec2, uv4: *ImVec2, col: ImU32 = 4294967295) -> void #cpp_method #foreign imgui "_ZN10ImDrawList12AddImageQuadEPvRK6ImVec2S3_S3_S3_S3_S3_S3_S3_j";
        AddImageQuad :: (this: *ImDrawList, user_texture_id: ImTextureID, p1: ImVec2, p2: ImVec2, p3: ImVec2, p4: ImVec2, uv1: ImVec2 = ImVec2.{ 0, 0 }, uv2: ImVec2 = ImVec2.{ 1, 0 }, uv3: ImVec2 = ImVec2.{ 1, 1 }, uv4: ImVec2 = ImVec2.{ 0, 1 }, col: ImU32 = 4294967295) #no_context {
            AddImageQuad(this, user_texture_id, *p1, *p2, *p3, *p4, *uv1, *uv2, *uv3, *uv4, col);
        }
        AddImageRounded :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: *ImVec2, p_max: *ImVec2, uv_min: *ImVec2, uv_max: *ImVec2, col: ImU32, rounding: float, flags: ImDrawFlags = .None) -> void #cpp_method #foreign imgui "_ZN10ImDrawList15AddImageRoundedEPvRK6ImVec2S3_S3_S3_jfi";
        AddImageRounded :: (this: *ImDrawList, user_texture_id: ImTextureID, p_min: ImVec2, p_max: ImVec2, uv_min: ImVec2, uv_max: ImVec2, col: ImU32, rounding: float, flags: ImDrawFlags = .None) #no_context {
            AddImageRounded(this, user_texture_id, *p_min, *p_max, *uv_min, *uv_max, col, rounding, flags);
        }
        PathArcTo :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList9PathArcToERK6ImVec2fffi";
        PathArcTo :: (this: *ImDrawList, center: ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32 = 0) #no_context {
            PathArcTo(this, *center, radius, a_min, a_max, num_segments);
        }
        PathArcToFast :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min_of_12: s32, a_max_of_12: s32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList13PathArcToFastERK6ImVec2fii";
        PathArcToFast :: (this: *ImDrawList, center: ImVec2, radius: float, a_min_of_12: s32, a_max_of_12: s32) #no_context {
            PathArcToFast(this, *center, radius, a_min_of_12, a_max_of_12);
        }
        PathBezierCubicCurveTo :: (this: *ImDrawList, p2: *ImVec2, p3: *ImVec2, p4: *ImVec2, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList22PathBezierCubicCurveToERK6ImVec2S2_S2_i";
        PathBezierCubicCurveTo :: (this: *ImDrawList, p2: ImVec2, p3: ImVec2, p4: ImVec2, num_segments: s32 = 0) #no_context {
            PathBezierCubicCurveTo(this, *p2, *p3, *p4, num_segments);
        }
        PathBezierQuadraticCurveTo :: (this: *ImDrawList, p2: *ImVec2, p3: *ImVec2, num_segments: s32 = 0) -> void #cpp_method #foreign imgui "_ZN10ImDrawList26PathBezierQuadraticCurveToERK6ImVec2S2_i";
        PathBezierQuadraticCurveTo :: (this: *ImDrawList, p2: ImVec2, p3: ImVec2, num_segments: s32 = 0) #no_context {
            PathBezierQuadraticCurveTo(this, *p2, *p3, num_segments);
        }
        PathRect :: (this: *ImDrawList, rect_min: *ImVec2, rect_max: *ImVec2, rounding: float = 0.0, flags: ImDrawFlags = .None) -> void #cpp_method #foreign imgui "_ZN10ImDrawList8PathRectERK6ImVec2S2_fi";
        PathRect :: (this: *ImDrawList, rect_min: ImVec2, rect_max: ImVec2, rounding: float = 0.0, flags: ImDrawFlags = .None) #no_context {
            PathRect(this, *rect_min, *rect_max, rounding, flags);
        }
        AddCallback :: (this: *ImDrawList, callback: ImDrawCallback, callback_data: *void) -> void #cpp_method #foreign imgui "_ZN10ImDrawList11AddCallbackEPFvPKS_PK9ImDrawCmdEPv";
        AddDrawCmd :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList10AddDrawCmdEv";
        CloneOutput :: (this: *ImDrawList) -> *ImDrawList #cpp_method #foreign imgui "_ZNK10ImDrawList11CloneOutputEv";
        PrimReserve :: (this: *ImDrawList, idx_count: s32, vtx_count: s32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList11PrimReserveEii";
        PrimUnreserve :: (this: *ImDrawList, idx_count: s32, vtx_count: s32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList13PrimUnreserveEii";
        PrimRect :: (this: *ImDrawList, a: *ImVec2, b: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList8PrimRectERK6ImVec2S2_j";
        PrimRect :: (this: *ImDrawList, a: ImVec2, b: ImVec2, col: ImU32) #no_context {
            PrimRect(this, *a, *b, col);
        }
        PrimRectUV :: (this: *ImDrawList, a: *ImVec2, b: *ImVec2, uv_a: *ImVec2, uv_b: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList10PrimRectUVERK6ImVec2S2_S2_S2_j";
        PrimRectUV :: (this: *ImDrawList, a: ImVec2, b: ImVec2, uv_a: ImVec2, uv_b: ImVec2, col: ImU32) #no_context {
            PrimRectUV(this, *a, *b, *uv_a, *uv_b, col);
        }
        PrimQuadUV :: (this: *ImDrawList, a: *ImVec2, b: *ImVec2, c: *ImVec2, d: *ImVec2, uv_a: *ImVec2, uv_b: *ImVec2, uv_c: *ImVec2, uv_d: *ImVec2, col: ImU32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList10PrimQuadUVERK6ImVec2S2_S2_S2_S2_S2_S2_S2_j";
        PrimQuadUV :: (this: *ImDrawList, a: ImVec2, b: ImVec2, c: ImVec2, d: ImVec2, uv_a: ImVec2, uv_b: ImVec2, uv_c: ImVec2, uv_d: ImVec2, col: ImU32) #no_context {
            PrimQuadUV(this, *a, *b, *c, *d, *uv_a, *uv_b, *uv_c, *uv_d, col);
        }
        _ResetForNewFrame :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList17_ResetForNewFrameEv";
        _ClearFreeMemory :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList16_ClearFreeMemoryEv";
        _PopUnusedDrawCmd :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList17_PopUnusedDrawCmdEv";
        _TryMergeDrawCmds :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList17_TryMergeDrawCmdsEv";
        _OnChangedClipRect :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList18_OnChangedClipRectEv";
        _OnChangedTextureID :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList19_OnChangedTextureIDEv";
        _OnChangedVtxOffset :: (this: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN10ImDrawList19_OnChangedVtxOffsetEv";
        _CalcCircleAutoSegmentCount :: (this: *ImDrawList, radius: float) -> s32 #cpp_method #foreign imgui "_ZNK10ImDrawList27_CalcCircleAutoSegmentCountEf";
        _PathArcToFastEx :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min_sample: s32, a_max_sample: s32, a_step: s32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList16_PathArcToFastExERK6ImVec2fiii";
        _PathArcToFastEx :: (this: *ImDrawList, center: ImVec2, radius: float, a_min_sample: s32, a_max_sample: s32, a_step: s32) #no_context {
            _PathArcToFastEx(this, *center, radius, a_min_sample, a_max_sample, a_step);
        }
        _PathArcToN :: (this: *ImDrawList, center: *ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32) -> void #cpp_method #foreign imgui "_ZN10ImDrawList11_PathArcToNERK6ImVec2fffi";
        _PathArcToN :: (this: *ImDrawList, center: ImVec2, radius: float, a_min: float, a_max: float, num_segments: s32) #no_context {
            _PathArcToN(this, *center, radius, a_min, a_max, num_segments);
        }
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImDrawListFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImDrawListSharedData :: struct {}

}

#if OS == .WINDOWS {
    ImDrawListSharedData :: struct {}

}

#if OS != .WINDOWS {
    ImDrawListSplitter :: struct {
        _Current: s32;
        _Count: s32;
        _Channels: ImVector(ImDrawChannel);
        ClearFreeMemory :: (this: *ImDrawListSplitter) -> void #cpp_method #foreign imgui "_ZN18ImDrawListSplitter15ClearFreeMemoryEv";
        Split :: (this: *ImDrawListSplitter, draw_list: *ImDrawList, count: s32) -> void #cpp_method #foreign imgui "_ZN18ImDrawListSplitter5SplitEP10ImDrawListi";
        Merge :: (this: *ImDrawListSplitter, draw_list: *ImDrawList) -> void #cpp_method #foreign imgui "_ZN18ImDrawListSplitter5MergeEP10ImDrawList";
        SetCurrentChannel :: (this: *ImDrawListSplitter, draw_list: *ImDrawList, channel_idx: s32) -> void #cpp_method #foreign imgui "_ZN18ImDrawListSplitter17SetCurrentChannelEP10ImDrawListi";
    }

}

#if OS == .WINDOWS {
    ImDrawListSplitter :: struct {
        _Current: s32;
        _Count: s32;
        _Channels: ImVector(ImDrawChannel);
        ClearFreeMemory :: (this: *ImDrawListSplitter) -> void #cpp_method #foreign imgui "?ClearFreeMemory@ImDrawListSplitter@@QEAAXXZ";
        Split :: (this: *ImDrawListSplitter, draw_list: *ImDrawList, count: s32) -> void #cpp_method #foreign imgui "?Split@ImDrawListSplitter@@QEAAXPEAUImDrawList@@H@Z";
        Merge :: (this: *ImDrawListSplitter, draw_list: *ImDrawList) -> void #cpp_method #foreign imgui "?Merge@ImDrawListSplitter@@QEAAXPEAUImDrawList@@@Z";
        SetCurrentChannel :: (this: *ImDrawListSplitter, draw_list: *ImDrawList, channel_idx: s32) -> void #cpp_method #foreign imgui "?SetCurrentChannel@ImDrawListSplitter@@QEAAXPEAUImDrawList@@H@Z";
    }

}

#if OS != .WINDOWS {
    ImDrawVert :: struct {
        pos: ImVec2;
        uv: ImVec2;
        col: ImU32;
    }

}

#if OS == .WINDOWS {
    ImDrawVert :: struct {
        pos: ImVec2;
        uv: ImVec2;
        col: ImU32;
    }

}

#if OS != .WINDOWS {
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
        Constructor :: (this: *ImFont) -> void #cpp_method #foreign imgui "_ZN6ImFontC1Ev";
        Destructor_Base :: (this: *ImFont) -> void #cpp_method #foreign imgui "_ZN6ImFontD2Ev";
        Destructor :: (this: *ImFont) -> void #cpp_method #foreign imgui "_ZN6ImFontD1Ev";
        FindGlyph :: (this: *ImFont, c: ImWchar) -> *ImFontGlyph #cpp_method #foreign imgui "_ZNK6ImFont9FindGlyphEt";
        FindGlyphNoFallback :: (this: *ImFont, c: ImWchar) -> *ImFontGlyph #cpp_method #foreign imgui "_ZNK6ImFont19FindGlyphNoFallbackEt";
        CalcTextSizeA :: (this: *ImFont, size: float, max_width: float, wrap_width: float, text_begin: *u8, text_end: *u8 = null, remaining: **u8 = null) -> ImVec2 #cpp_method #foreign imgui "_ZNK6ImFont13CalcTextSizeAEfffPKcS1_PS1_";
        CalcTextSizeA :: (this: *ImFont, size: float, max_width: float, wrap_width: float, text: string, remaining: **u8 = null) -> ImVec2 #no_context {
            return CalcTextSizeA(this, size, max_width, wrap_width, text.data, text.data + text.count, remaining);
        }
        CalcWordWrapPositionA :: (this: *ImFont, scale: float, text: *u8, text_end: *u8, wrap_width: float) -> *u8 #cpp_method #foreign imgui "_ZNK6ImFont21CalcWordWrapPositionAEfPKcS1_f";
        CalcWordWrapPositionA :: (this: *ImFont, scale: float, text: string, wrap_width: float) -> *u8 #no_context {
            return CalcWordWrapPositionA(this, scale, text.data, text.data + text.count, wrap_width);
        }
        RenderChar :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: *ImVec2, col: ImU32, c: ImWchar) -> void #cpp_method #foreign imgui "_ZNK6ImFont10RenderCharEP10ImDrawListfRK6ImVec2jt";
        RenderChar :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: ImVec2, col: ImU32, c: ImWchar) #no_context {
            RenderChar(this, draw_list, size, *pos, col, c);
        }
        RenderText :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: *ImVec2, col: ImU32, clip_rect: *ImVec4, text_begin: *u8, text_end: *u8, wrap_width: float = 0.0, cpu_fine_clip := false) -> void #cpp_method #foreign imgui "_ZNK6ImFont10RenderTextEP10ImDrawListfRK6ImVec2jRK6ImVec4PKcS9_fb";
        RenderText :: (this: *ImFont, draw_list: *ImDrawList, size: float, pos: ImVec2, col: ImU32, clip_rect: ImVec4, text: string, wrap_width: float = 0.0, cpu_fine_clip := false) #no_context {
            RenderText(this, draw_list, size, *pos, col, *clip_rect, text.data, text.data + text.count, wrap_width, cpu_fine_clip);
        }
        BuildLookupTable :: (this: *ImFont) -> void #cpp_method #foreign imgui "_ZN6ImFont16BuildLookupTableEv";
        ClearOutputData :: (this: *ImFont) -> void #cpp_method #foreign imgui "_ZN6ImFont15ClearOutputDataEv";
        GrowIndex :: (this: *ImFont, new_size: s32) -> void #cpp_method #foreign imgui "_ZN6ImFont9GrowIndexEi";
        AddGlyph :: (this: *ImFont, src_cfg: *ImFontConfig, c: ImWchar, x0: float, y0: float, x1: float, y1: float, u0: float, v0: float, u1: float, v1: float, advance_x: float) -> void #cpp_method #foreign imgui "_ZN6ImFont8AddGlyphEPK12ImFontConfigtfffffffff";
        AddRemapChar :: (this: *ImFont, dst: ImWchar, src: ImWchar, overwrite_dst := true) -> void #cpp_method #foreign imgui "_ZN6ImFont12AddRemapCharEttb";
        SetGlyphVisible :: (this: *ImFont, c: ImWchar, visible: bool) -> void #cpp_method #foreign imgui "_ZN6ImFont15SetGlyphVisibleEtb";
        IsGlyphRangeUnused :: (this: *ImFont, c_begin: u32, c_last: u32) -> bool #cpp_method #foreign imgui "_ZN6ImFont18IsGlyphRangeUnusedEjj";
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImFontAtlas :: struct {
        Constructor :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlasC1Ev";
        Destructor_Base :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlasD2Ev";
        Destructor :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlasD1Ev";
        AddFont :: (this: *ImFontAtlas, font_cfg: *ImFontConfig) -> *ImFont #cpp_method #foreign imgui "_ZN11ImFontAtlas7AddFontEPK12ImFontConfig";
        AddFontDefault :: (this: *ImFontAtlas, font_cfg: *ImFontConfig = null) -> *ImFont #cpp_method #foreign imgui "_ZN11ImFontAtlas14AddFontDefaultEPK12ImFontConfig";
        AddFontFromFileTTF :: (this: *ImFontAtlas, filename: *u8, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "_ZN11ImFontAtlas18AddFontFromFileTTFEPKcfPK12ImFontConfigPKt";
        AddFontFromMemoryTTF :: (this: *ImFontAtlas, font_data: *void, font_size: s32, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "_ZN11ImFontAtlas20AddFontFromMemoryTTFEPvifPK12ImFontConfigPKt";
        AddFontFromMemoryCompressedTTF :: (this: *ImFontAtlas, compressed_font_data: *void, compressed_font_size: s32, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "_ZN11ImFontAtlas30AddFontFromMemoryCompressedTTFEPKvifPK12ImFontConfigPKt";
        AddFontFromMemoryCompressedBase85TTF :: (this: *ImFontAtlas, compressed_font_data_base85: *u8, size_pixels: float, font_cfg: *ImFontConfig = null, glyph_ranges: *ImWchar = null) -> *ImFont #cpp_method #foreign imgui "_ZN11ImFontAtlas36AddFontFromMemoryCompressedBase85TTFEPKcfPK12ImFontConfigPKt";
        ClearInputData :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlas14ClearInputDataEv";
        ClearTexData :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlas12ClearTexDataEv";
        ClearFonts :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlas10ClearFontsEv";
        Clear :: (this: *ImFontAtlas) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlas5ClearEv";
        Build :: (this: *ImFontAtlas) -> bool #cpp_method #foreign imgui "_ZN11ImFontAtlas5BuildEv";
        GetTexDataAsAlpha8 :: (this: *ImFontAtlas, out_pixels: **u8, out_width: *s32, out_height: *s32, out_bytes_per_pixel: *s32 = null) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlas18GetTexDataAsAlpha8EPPhPiS2_S2_";
        GetTexDataAsRGBA32 :: (this: *ImFontAtlas, out_pixels: **u8, out_width: *s32, out_height: *s32, out_bytes_per_pixel: *s32 = null) -> void #cpp_method #foreign imgui "_ZN11ImFontAtlas18GetTexDataAsRGBA32EPPhPiS2_S2_";
        GetGlyphRangesDefault :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas21GetGlyphRangesDefaultEv";
        GetGlyphRangesGreek :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas19GetGlyphRangesGreekEv";
        GetGlyphRangesKorean :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas20GetGlyphRangesKoreanEv";
        GetGlyphRangesJapanese :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas22GetGlyphRangesJapaneseEv";
        GetGlyphRangesChineseFull :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas25GetGlyphRangesChineseFullEv";
        GetGlyphRangesChineseSimplifiedCommon :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas37GetGlyphRangesChineseSimplifiedCommonEv";
        GetGlyphRangesCyrillic :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas22GetGlyphRangesCyrillicEv";
        GetGlyphRangesThai :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas18GetGlyphRangesThaiEv";
        GetGlyphRangesVietnamese :: (this: *ImFontAtlas) -> *ImWchar #cpp_method #foreign imgui "_ZN11ImFontAtlas24GetGlyphRangesVietnameseEv";
        AddCustomRectRegular :: (this: *ImFontAtlas, width: s32, height: s32) -> s32 #cpp_method #foreign imgui "_ZN11ImFontAtlas20AddCustomRectRegularEii";
        AddCustomRectFontGlyph :: (this: *ImFontAtlas, font: *ImFont, id: ImWchar, width: s32, height: s32, advance_x: float, offset: *ImVec2) -> s32 #cpp_method #foreign imgui "_ZN11ImFontAtlas22AddCustomRectFontGlyphEP6ImFonttiifRK6ImVec2";
        AddCustomRectFontGlyph :: (this: *ImFontAtlas, font: *ImFont, id: ImWchar, width: s32, height: s32, advance_x: float, offset: ImVec2 = ImVec2.{ 0, 0 }) -> s32 #no_context {
            return AddCustomRectFontGlyph(this, font, id, width, height, advance_x, *offset);
        }
        CalcCustomRectUV :: (this: *ImFontAtlas, rect: *ImFontAtlasCustomRect, out_uv_min: *ImVec2, out_uv_max: *ImVec2) -> void #cpp_method #foreign imgui "_ZNK11ImFontAtlas16CalcCustomRectUVEPK21ImFontAtlasCustomRectP6ImVec2S4_";
        GetMouseCursorTexData :: (this: *ImFontAtlas, cursor: MouseCursor, out_offset: *ImVec2, out_size: *ImVec2, out_uv_border: *[2] ImVec2, out_uv_fill: *[2] ImVec2) -> bool #cpp_method #foreign imgui "_ZN11ImFontAtlas21GetMouseCursorTexDataEiP6ImVec2S1_S1_S1_";
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImFontAtlasFlags :: enum_flags u32 {
        None :: 0x0;
        NoPowerOfTwoHeight :: 0x1;
        NoMouseCursors :: 0x2;
        NoBakedLines :: 0x4;
        ImFontAtlasFlags_None :: None;
        ImFontAtlasFlags_NoPowerOfTwoHeight :: NoPowerOfTwoHeight;
        ImFontAtlasFlags_NoMouseCursors :: NoMouseCursors;
        ImFontAtlasFlags_NoBakedLines :: NoBakedLines;
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImFontBuilderIO :: struct {}

}

#if OS == .WINDOWS {
    ImFontBuilderIO :: struct {}

}

#if OS != .WINDOWS {
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
        Constructor :: (this: *ImFontConfig) -> void #cpp_method #foreign imgui "_ZN12ImFontConfigC1Ev";
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ImFontGlyph :: struct {
        __bitfield: u32;
        Visible: u32;
        Codepoint: u32;
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

}

#if OS == .WINDOWS {
    ImFontGlyph :: struct {
        __bitfield: u32;
        Visible: u32;
        Codepoint: u32;
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

}

#if OS != .WINDOWS {
    ImFontGlyphRangesBuilder :: struct {
        UsedChars: ImVector(ImU32);
        AddText :: (this: *ImFontGlyphRangesBuilder, text: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "_ZN24ImFontGlyphRangesBuilder7AddTextEPKcS1_";
        AddText :: (this: *ImFontGlyphRangesBuilder, text: string) #no_context {
            AddText(this, text.data, text.data + text.count);
        }
        AddRanges :: (this: *ImFontGlyphRangesBuilder, ranges: *ImWchar) -> void #cpp_method #foreign imgui "_ZN24ImFontGlyphRangesBuilder9AddRangesEPKt";
        BuildRanges :: (this: *ImFontGlyphRangesBuilder, out_ranges: *ImVector(ImWchar)) -> void #cpp_method #foreign imgui "_ZN24ImFontGlyphRangesBuilder11BuildRangesEP8ImVectorItE";
    }

}

#if OS == .WINDOWS {
    ImFontGlyphRangesBuilder :: struct {
        UsedChars: ImVector(ImU32);
        AddText :: (this: *ImFontGlyphRangesBuilder, text: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "?AddText@ImFontGlyphRangesBuilder@@QEAAXPEBD0@Z";
        AddText :: (this: *ImFontGlyphRangesBuilder, text: string) #no_context {
            AddText(this, text.data, text.data + text.count);
        }
        AddRanges :: (this: *ImFontGlyphRangesBuilder, ranges: *ImWchar) -> void #cpp_method #foreign imgui "?AddRanges@ImFontGlyphRangesBuilder@@QEAAXPEBG@Z";
        BuildRanges :: (this: *ImFontGlyphRangesBuilder, out_ranges: *ImVector(ImWchar)) -> void #cpp_method #foreign imgui "?BuildRanges@ImFontGlyphRangesBuilder@@QEAAXPEAU?$ImVector@G@@@Z";
    }

}

#if OS != .WINDOWS {
    ImGuiContext :: struct {}

}

#if OS == .WINDOWS {
    ImGuiContext :: struct {}

}

#if OS != .WINDOWS {
    ImNewWrapper :: struct {
        __empty_struct_padding: u8;
    }

}

#if OS == .WINDOWS {
    ImNewWrapper :: struct {
        __empty_struct_padding: u8;
    }

}

ImVector :: struct(T: Type) {
    Size: s32;
    Capacity: s32;
    Data: *T;
}

#if OS != .WINDOWS {
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
        Constructor :: (this: *InputTextCallbackData) -> void #cpp_method #foreign imgui "_ZN26ImGuiInputTextCallbackDataC1Ev";
        DeleteChars :: (this: *InputTextCallbackData, pos: s32, bytes_count: s32) -> void #cpp_method #foreign imgui "_ZN26ImGuiInputTextCallbackData11DeleteCharsEii";
        InsertChars :: (this: *InputTextCallbackData, pos: s32, text: *u8, text_end: *u8 = null) -> void #cpp_method #foreign imgui "_ZN26ImGuiInputTextCallbackData11InsertCharsEiPKcS1_";
        InsertChars :: (this: *InputTextCallbackData, pos: s32, text: string) #no_context {
            InsertChars(this, pos, text.data, text.data + text.count);
        }
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    InputTextFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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
        AddKeyEvent :: (this: *IO, key: Key, down: bool) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO11AddKeyEventE8ImGuiKeyb";
        AddKeyAnalogEvent :: (this: *IO, key: Key, down: bool, v: float) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO17AddKeyAnalogEventE8ImGuiKeybf";
        AddMousePosEvent :: (this: *IO, x: float, y: float) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO16AddMousePosEventEff";
        AddMouseButtonEvent :: (this: *IO, button: s32, down: bool) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO19AddMouseButtonEventEib";
        AddMouseWheelEvent :: (this: *IO, wheel_x: float, wheel_y: float) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO18AddMouseWheelEventEff";
        AddMouseSourceEvent :: (this: *IO, source: MouseSource) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO19AddMouseSourceEventE16ImGuiMouseSource";
        AddFocusEvent :: (this: *IO, focused: bool) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO13AddFocusEventEb";
        AddInputCharacter :: (this: *IO, c: u32) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO17AddInputCharacterEj";
        AddInputCharacterUTF16 :: (this: *IO, c: ImWchar16) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO22AddInputCharacterUTF16Et";
        AddInputCharactersUTF8 :: (this: *IO, str: *u8) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO22AddInputCharactersUTF8EPKc";
        SetKeyEventNativeData :: (this: *IO, key: Key, native_keycode: s32, native_scancode: s32, native_legacy_index: s32 = -1) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO21SetKeyEventNativeDataE8ImGuiKeyiii";
        SetAppAcceptingEvents :: (this: *IO, accepting_events: bool) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO21SetAppAcceptingEventsEb";
        ClearInputCharacters :: (this: *IO) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO20ClearInputCharactersEv";
        ClearInputKeys :: (this: *IO) -> void #cpp_method #foreign imgui "_ZN7ImGuiIO14ClearInputKeysEv";
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
        Constructor :: (this: *IO) -> void #cpp_method #foreign imgui "_ZN7ImGuiIOC1Ev";
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    KeyData :: struct {
        Down: bool;
        DownDuration: float;
        DownDurationPrev: float;
        AnalogValue: float;
    }

}

#if OS == .WINDOWS {
    KeyData :: struct {
        Down: bool;
        DownDuration: float;
        DownDurationPrev: float;
        AnalogValue: float;
    }

}

#if OS != .WINDOWS {
    ListClipper :: struct {
        Ctx: *ImGuiContext;
        DisplayStart: s32;
        DisplayEnd: s32;
        ItemsCount: s32;
        ItemsHeight: float;
        StartPosY: float;
        TempData: *void;
        Constructor :: (this: *ListClipper) -> void #cpp_method #foreign imgui "_ZN16ImGuiListClipperC1Ev";
        Destructor_Base :: (this: *ListClipper) -> void #cpp_method #foreign imgui "_ZN16ImGuiListClipperD2Ev";
        Destructor :: (this: *ListClipper) -> void #cpp_method #foreign imgui "_ZN16ImGuiListClipperD1Ev";
        Begin :: (this: *ListClipper, items_count: s32, items_height: float = -1.0) -> void #cpp_method #foreign imgui "_ZN16ImGuiListClipper5BeginEif";
        End :: (this: *ListClipper) -> void #cpp_method #foreign imgui "_ZN16ImGuiListClipper3EndEv";
        Step :: (this: *ListClipper) -> bool #cpp_method #foreign imgui "_ZN16ImGuiListClipper4StepEv";
        IncludeRangeByIndices :: (this: *ListClipper, item_begin: s32, item_end: s32) -> void #cpp_method #foreign imgui "_ZN16ImGuiListClipper21IncludeRangeByIndicesEii";
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    ModFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    MouseButton :: enum u32 {
        Left :: 0;
        Right :: 1;
        Middle :: 2;
        COUNT :: 5;
        ImGuiMouseButton_Left :: Left;
        ImGuiMouseButton_Right :: Right;
        ImGuiMouseButton_Middle :: Middle;
        ImGuiMouseButton_COUNT :: COUNT;
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    NavInput :: enum u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    OnceUponAFrame :: struct {
        RefFrame: s32;
    }

}

#if OS == .WINDOWS {
    OnceUponAFrame :: struct {
        RefFrame: s32;
    }

}

#if OS != .WINDOWS {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    PlatformImeData :: struct {
        WantVisible: bool;
        InputPos: ImVec2;
        InputLineHeight: float;
    }

}

#if OS == .WINDOWS {
    PlatformImeData :: struct {
        WantVisible: bool;
        InputPos: ImVec2;
        InputLineHeight: float;
    }

}

#if OS != .WINDOWS {
    PopupFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    SelectableFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    SizeCallbackData :: struct {
        UserData: *void;
        Pos: ImVec2;
        CurrentSize: ImVec2;
        DesiredSize: ImVec2;
    }

}

#if OS == .WINDOWS {
    SizeCallbackData :: struct {
        UserData: *void;
        Pos: ImVec2;
        CurrentSize: ImVec2;
        DesiredSize: ImVec2;
    }

}

#if OS != .WINDOWS {
    SliderFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    SortDirection :: enum u32 {
        None :: 0;
        Ascending :: 1;
        Descending :: 2;
        ImGuiSortDirection_None :: None;
        ImGuiSortDirection_Ascending :: Ascending;
        ImGuiSortDirection_Descending :: Descending;
    }

}

#if OS == .WINDOWS {
    SortDirection :: enum s32 {
        None :: 0;
        Ascending :: 1;
        Descending :: 2;
        ImGuiSortDirection_None :: None;
        ImGuiSortDirection_Ascending :: Ascending;
        ImGuiSortDirection_Descending :: Descending;
    }

}

#if OS != .WINDOWS {
    Storage :: struct {
        StoragePair :: struct {
            key: ID;
            val_p: *void;
            #overlay (val_p) val_i: s32;
            #overlay (val_p) val_f: float;
        };
        Data: ImVector(StoragePair);
        GetInt :: (this: *Storage, key: ID, default_val: s32 = 0) -> s32 #cpp_method #foreign imgui "_ZNK12ImGuiStorage6GetIntEji";
        SetInt :: (this: *Storage, key: ID, val: s32) -> void #cpp_method #foreign imgui "_ZN12ImGuiStorage6SetIntEji";
        GetBool :: (this: *Storage, key: ID, default_val := false) -> bool #cpp_method #foreign imgui "_ZNK12ImGuiStorage7GetBoolEjb";
        SetBool :: (this: *Storage, key: ID, val: bool) -> void #cpp_method #foreign imgui "_ZN12ImGuiStorage7SetBoolEjb";
        GetFloat :: (this: *Storage, key: ID, default_val: float = 0.0) -> float #cpp_method #foreign imgui "_ZNK12ImGuiStorage8GetFloatEjf";
        SetFloat :: (this: *Storage, key: ID, val: float) -> void #cpp_method #foreign imgui "_ZN12ImGuiStorage8SetFloatEjf";
        GetVoidPtr :: (this: *Storage, key: ID) -> *void #cpp_method #foreign imgui "_ZNK12ImGuiStorage10GetVoidPtrEj";
        SetVoidPtr :: (this: *Storage, key: ID, val: *void) -> void #cpp_method #foreign imgui "_ZN12ImGuiStorage10SetVoidPtrEjPv";
        GetIntRef :: (this: *Storage, key: ID, default_val: s32 = 0) -> *s32 #cpp_method #foreign imgui "_ZN12ImGuiStorage9GetIntRefEji";
        GetBoolRef :: (this: *Storage, key: ID, default_val := false) -> *bool #cpp_method #foreign imgui "_ZN12ImGuiStorage10GetBoolRefEjb";
        GetFloatRef :: (this: *Storage, key: ID, default_val: float = 0.0) -> *float #cpp_method #foreign imgui "_ZN12ImGuiStorage11GetFloatRefEjf";
        GetVoidPtrRef :: (this: *Storage, key: ID, default_val: *void = null) -> **void #cpp_method #foreign imgui "_ZN12ImGuiStorage13GetVoidPtrRefEjPv";
        SetAllInt :: (this: *Storage, val: s32) -> void #cpp_method #foreign imgui "_ZN12ImGuiStorage9SetAllIntEi";
        BuildSortByKey :: (this: *Storage) -> void #cpp_method #foreign imgui "_ZN12ImGuiStorage14BuildSortByKeyEv";
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
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
        Constructor :: (this: *Style) -> void #cpp_method #foreign imgui "_ZN10ImGuiStyleC1Ev";
        ScaleAllSizes :: (this: *Style, scale_factor: float) -> void #cpp_method #foreign imgui "_ZN10ImGuiStyle13ScaleAllSizesEf";
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    StyleVar :: enum u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TabBarFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TabItemFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TableBgTarget :: enum u32 {
        None :: 0;
        RowBg0 :: 1;
        RowBg1 :: 2;
        CellBg :: 3;
        ImGuiTableBgTarget_None :: None;
        ImGuiTableBgTarget_RowBg0 :: RowBg0;
        ImGuiTableBgTarget_RowBg1 :: RowBg1;
        ImGuiTableBgTarget_CellBg :: CellBg;
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TableColumnFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TableColumnSortSpecs :: struct {
        ColumnUserID: ID;
        ColumnIndex: ImS16;
        SortOrder: ImS16;
        SortDirection: u8;
    }

}

#if OS == .WINDOWS {
    TableColumnSortSpecs :: struct {
        ColumnUserID: ID;
        ColumnIndex: ImS16;
        SortOrder: ImS16;
        SortDirection_: SortDirection;
    }

}

#if OS != .WINDOWS {
    TableFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TableRowFlags :: enum_flags u32 {
        None :: 0x0;
        Headers :: 0x1;
        ImGuiTableRowFlags_None :: None;
        ImGuiTableRowFlags_Headers :: Headers;
    }

}

#if OS == .WINDOWS {
    TableRowFlags :: enum_flags s32 {
        None :: 0x0;
        Headers :: 0x1;
        ImGuiTableRowFlags_None :: None;
        ImGuiTableRowFlags_Headers :: Headers;
    }

}

#if OS != .WINDOWS {
    TableSortSpecs :: struct {
        Specs: *TableColumnSortSpecs;
        SpecsCount: s32;
        SpecsDirty: bool;
    }

}

#if OS == .WINDOWS {
    TableSortSpecs :: struct {
        Specs: *TableColumnSortSpecs;
        SpecsCount: s32;
        SpecsDirty: bool;
    }

}

#if OS != .WINDOWS {
    TextFilter :: struct {
        Constructor :: (this: *TextFilter, default_filter: *u8 = "") -> void #cpp_method #foreign imgui "_ZN15ImGuiTextFilterC1EPKc";
        Draw :: (this: *TextFilter, label: *u8 = "Filter (inc,-exc)", width: float = 0.0) -> bool #cpp_method #foreign imgui "_ZN15ImGuiTextFilter4DrawEPKcf";
        PassFilter :: (this: *TextFilter, text: *u8, text_end: *u8 = null) -> bool #cpp_method #foreign imgui "_ZNK15ImGuiTextFilter10PassFilterEPKcS1_";
        PassFilter :: (this: *TextFilter, text: string) -> bool #no_context {
            return PassFilter(this, text.data, text.data + text.count);
        }
        Build :: (this: *TextFilter) -> void #cpp_method #foreign imgui "_ZN15ImGuiTextFilter5BuildEv";
        TextRange :: struct {
            b: *u8;
            e: *u8;
            split :: (this: *TextRange, separator: u8, out: *ImVector(TextRange)) -> void #cpp_method #foreign imgui "_ZNK15ImGuiTextFilter14ImGuiTextRange5splitEcP8ImVectorIS0_E";
        };
        InputBuf: [256] u8;
        Filters: ImVector(TextRange);
        CountGrep: s32;
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    TreeNodeFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    Viewport :: struct {
        Flags: ViewportFlags;
        Pos: ImVec2;
        Size: ImVec2;
        WorkPos: ImVec2;
        WorkSize: ImVec2;
        PlatformHandleRaw: *void;
    }

}

#if OS == .WINDOWS {
    Viewport :: struct {
        Flags: ViewportFlags;
        Pos: ImVec2;
        Size: ImVec2;
        WorkPos: ImVec2;
        WorkSize: ImVec2;
        PlatformHandleRaw: *void;
    }

}

#if OS != .WINDOWS {
    ViewportFlags :: enum_flags u32 {
        None :: 0x0;
        IsPlatformWindow :: 0x1;
        IsPlatformMonitor :: 0x2;
        OwnedByApp :: 0x4;
        ImGuiViewportFlags_None :: None;
        ImGuiViewportFlags_IsPlatformWindow :: IsPlatformWindow;
        ImGuiViewportFlags_IsPlatformMonitor :: IsPlatformMonitor;
        ImGuiViewportFlags_OwnedByApp :: OwnedByApp;
    }

}

#if OS == .WINDOWS {
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

}

#if OS != .WINDOWS {
    WindowFlags :: enum_flags u32 {
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

}

#if OS == .WINDOWS {
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

}

// Callback and opaque types

ImDrawCallback :: #type (parent_list: *ImDrawList, cmd: *ImDrawCmd) #c_call;

#if OS != .WINDOWS {
    InputTextCallback :: #type (data: *InputTextCallbackData) -> s32 #c_call;
}

#if OS == .WINDOWS {
    InputTextCallback :: #type (data: *InputTextCallbackData) -> s32 #c_call;
}

#if OS != .WINDOWS {
    MemAllocFunc :: #type (sz: u64, user_data: *void) -> *void #c_call;
}

#if OS == .WINDOWS {
    MemAllocFunc :: #type (sz: u64, user_data: *void) -> *void #c_call;
}

#if OS != .WINDOWS {
    MemFreeFunc :: #type (ptr: *void, user_data: *void) -> void #c_call;
}

#if OS == .WINDOWS {
    MemFreeFunc :: #type (ptr: *void, user_data: *void) -> void #c_call;
}

#if OS != .WINDOWS {
    SizeCallback :: #type (data: *SizeCallbackData) -> void #c_call;
}

#if OS == .WINDOWS {
    SizeCallback :: #type (data: *SizeCallbackData) -> void #c_call;
}

// Aliases and constants

#if OS != .WINDOWS {
    ID :: u32;
}

#if OS == .WINDOWS {
    ID :: u32;
}

#if OS != .WINDOWS {
    IM_COL32_A_MASK :: 0xFF000000;
}

#if OS == .WINDOWS {
    IM_COL32_A_MASK :: 0xFF000000;
}

#if OS != .WINDOWS {
    IM_COL32_A_SHIFT :: 24;
}

#if OS == .WINDOWS {
    IM_COL32_A_SHIFT :: 24;
}

#if OS != .WINDOWS {
    IM_COL32_B_SHIFT :: 16;
}

#if OS == .WINDOWS {
    IM_COL32_B_SHIFT :: 16;
}

#if OS != .WINDOWS {
    IM_COL32_G_SHIFT :: 8;
}

#if OS == .WINDOWS {
    IM_COL32_G_SHIFT :: 8;
}

#if OS != .WINDOWS {
    IM_COL32_R_SHIFT :: 0;
}

#if OS == .WINDOWS {
    IM_COL32_R_SHIFT :: 0;
}

#if OS != .WINDOWS {
    IM_DRAWLIST_TEX_LINES_WIDTH_MAX :: 63;
}

#if OS == .WINDOWS {
    IM_DRAWLIST_TEX_LINES_WIDTH_MAX :: 63;
}

#if OS != .WINDOWS {
    IM_UNICODE_CODEPOINT_INVALID :: 0xFFFD;
}

#if OS == .WINDOWS {
    IM_UNICODE_CODEPOINT_INVALID :: 0xFFFD;
}

#if OS != .WINDOWS {
    IM_UNICODE_CODEPOINT_MAX :: 0xFFFF;
}

#if OS == .WINDOWS {
    IM_UNICODE_CODEPOINT_MAX :: 0xFFFF;
}

ImColor :: Vector4;

#if OS != .WINDOWS {
    ImDrawIdx :: u16;
}

#if OS == .WINDOWS {
    ImDrawIdx :: u16;
}

#if OS != .WINDOWS {
    IMGUI_PAYLOAD_TYPE_COLOR_3F :: "_COL3F";
}

#if OS == .WINDOWS {
    IMGUI_PAYLOAD_TYPE_COLOR_3F :: "_COL3F";
}

#if OS != .WINDOWS {
    IMGUI_PAYLOAD_TYPE_COLOR_4F :: "_COL4F";
}

#if OS == .WINDOWS {
    IMGUI_PAYLOAD_TYPE_COLOR_4F :: "_COL4F";
}

#if OS != .WINDOWS {
    IMGUI_VERSION :: "1.89.6";
}

#if OS == .WINDOWS {
    IMGUI_VERSION :: "1.89.6";
}

#if OS != .WINDOWS {
    IMGUI_VERSION_NUM :: 18960;
}

#if OS == .WINDOWS {
    IMGUI_VERSION_NUM :: 18960;
}

#if OS != .WINDOWS {
    ImS16 :: s16;
}

#if OS == .WINDOWS {
    ImS16 :: s16;
}

#if OS != .WINDOWS {
    ImS32 :: s32;
}

#if OS == .WINDOWS {
    ImS32 :: s32;
}

#if OS != .WINDOWS {
    ImS64 :: s64;
}

#if OS == .WINDOWS {
    ImS64 :: s64;
}

#if OS != .WINDOWS {
    ImS8 :: s8;
}

#if OS == .WINDOWS {
    ImS8 :: s8;
}

#if OS != .WINDOWS {
    ImTextureID :: *void;
}

#if OS == .WINDOWS {
    ImTextureID :: *void;
}

#if OS != .WINDOWS {
    ImU16 :: u16;
}

#if OS == .WINDOWS {
    ImU16 :: u16;
}

#if OS != .WINDOWS {
    ImU32 :: u32;
}

#if OS == .WINDOWS {
    ImU32 :: u32;
}

#if OS != .WINDOWS {
    ImU64 :: u64;
}

#if OS == .WINDOWS {
    ImU64 :: u64;
}

#if OS != .WINDOWS {
    ImU8 :: u8;
}

#if OS == .WINDOWS {
    ImU8 :: u8;
}

ImVec2 :: Vector2;

ImVec4 :: Vector4;

#if OS != .WINDOWS {
    ImWchar :: ImWchar16;
}

#if OS == .WINDOWS {
    ImWchar :: ImWchar16;
}

#if OS != .WINDOWS {
    ImWchar16 :: u16;
}

#if OS == .WINDOWS {
    ImWchar16 :: u16;
}

#if OS != .WINDOWS {
    ImWchar32 :: u32;
}

#if OS == .WINDOWS {
    ImWchar32 :: u32;
}

#if OS != .WINDOWS {
    KeyChord :: s32;
}

#if OS == .WINDOWS {
    KeyChord :: s32;
}

// Native entry points

#if OS != .WINDOWS {
    AcceptDragDropPayload :: (type: *u8, flags: DragDropFlags = .None) -> *Payload #foreign imgui "_ZN5ImGui21AcceptDragDropPayloadEPKci";
}

#if OS == .WINDOWS {
    AcceptDragDropPayload :: (type: *u8, flags: DragDropFlags = .None) -> *Payload #foreign imgui "?AcceptDragDropPayload@ImGui@@YAPEBUImGuiPayload@@PEBDH@Z";
}

#if OS != .WINDOWS {
    AlignTextToFramePadding :: () -> void #foreign imgui "_ZN5ImGui23AlignTextToFramePaddingEv";
}

#if OS == .WINDOWS {
    AlignTextToFramePadding :: () -> void #foreign imgui "?AlignTextToFramePadding@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    ArrowButton :: (str_id: *u8, dir: Dir) -> bool #foreign imgui "_ZN5ImGui11ArrowButtonEPKci";
}

#if OS == .WINDOWS {
    ArrowButton :: (str_id: *u8, dir: Dir) -> bool #foreign imgui "?ArrowButton@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    Begin :: (name: *u8, p_open: *bool = null, flags: WindowFlags = .None) -> bool #foreign imgui "_ZN5ImGui5BeginEPKcPbi";
}

#if OS == .WINDOWS {
    Begin :: (name: *u8, p_open: *bool = null, flags: WindowFlags = .None) -> bool #foreign imgui "?Begin@ImGui@@YA_NPEBDPEA_NH@Z";
}

#if OS != .WINDOWS {
    BeginChild :: (str_id: *u8, size: *ImVec2, border := false, flags: WindowFlags = .None) -> bool #foreign imgui "_ZN5ImGui10BeginChildEPKcRK6ImVec2bi";
}

#if OS != .WINDOWS {
    BeginChild :: (id: ID, size: *ImVec2, border := false, flags: WindowFlags = .None) -> bool #foreign imgui "_ZN5ImGui10BeginChildEjRK6ImVec2bi";
}

#if OS == .WINDOWS {
    BeginChild :: (str_id: *u8, size: *ImVec2, border := false, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginChild@ImGui@@YA_NPEBDAEBUImVec2@@_NH@Z";
}

#if OS == .WINDOWS {
    BeginChild :: (id: ID, size: *ImVec2, border := false, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginChild@ImGui@@YA_NIAEBUImVec2@@_NH@Z";
}

#if OS != .WINDOWS {
    BeginChildFrame :: (id: ID, size: *ImVec2, flags: WindowFlags = .None) -> bool #foreign imgui "_ZN5ImGui15BeginChildFrameEjRK6ImVec2i";
}

#if OS == .WINDOWS {
    BeginChildFrame :: (id: ID, size: *ImVec2, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginChildFrame@ImGui@@YA_NIAEBUImVec2@@H@Z";
}

#if OS != .WINDOWS {
    BeginCombo :: (label: *u8, preview_value: *u8, flags: ComboFlags = .None) -> bool #foreign imgui "_ZN5ImGui10BeginComboEPKcS1_i";
}

#if OS == .WINDOWS {
    BeginCombo :: (label: *u8, preview_value: *u8, flags: ComboFlags = .None) -> bool #foreign imgui "?BeginCombo@ImGui@@YA_NPEBD0H@Z";
}

#if OS != .WINDOWS {
    BeginDisabled :: (disabled := true) -> void #foreign imgui "_ZN5ImGui13BeginDisabledEb";
}

#if OS == .WINDOWS {
    BeginDisabled :: (disabled := true) -> void #foreign imgui "?BeginDisabled@ImGui@@YAX_N@Z";
}

#if OS != .WINDOWS {
    BeginDragDropSource :: (flags: DragDropFlags = .None) -> bool #foreign imgui "_ZN5ImGui19BeginDragDropSourceEi";
}

#if OS == .WINDOWS {
    BeginDragDropSource :: (flags: DragDropFlags = .None) -> bool #foreign imgui "?BeginDragDropSource@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    BeginDragDropTarget :: () -> bool #foreign imgui "_ZN5ImGui19BeginDragDropTargetEv";
}

#if OS == .WINDOWS {
    BeginDragDropTarget :: () -> bool #foreign imgui "?BeginDragDropTarget@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    BeginGroup :: () -> void #foreign imgui "_ZN5ImGui10BeginGroupEv";
}

#if OS == .WINDOWS {
    BeginGroup :: () -> void #foreign imgui "?BeginGroup@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    BeginListBox :: (label: *u8, size: *ImVec2) -> bool #foreign imgui "_ZN5ImGui12BeginListBoxEPKcRK6ImVec2";
}

#if OS == .WINDOWS {
    BeginListBox :: (label: *u8, size: *ImVec2) -> bool #foreign imgui "?BeginListBox@ImGui@@YA_NPEBDAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    BeginMainMenuBar :: () -> bool #foreign imgui "_ZN5ImGui16BeginMainMenuBarEv";
}

#if OS == .WINDOWS {
    BeginMainMenuBar :: () -> bool #foreign imgui "?BeginMainMenuBar@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    BeginMenu :: (label: *u8, enabled := true) -> bool #foreign imgui "_ZN5ImGui9BeginMenuEPKcb";
}

#if OS == .WINDOWS {
    BeginMenu :: (label: *u8, enabled := true) -> bool #foreign imgui "?BeginMenu@ImGui@@YA_NPEBD_N@Z";
}

#if OS != .WINDOWS {
    BeginMenuBar :: () -> bool #foreign imgui "_ZN5ImGui12BeginMenuBarEv";
}

#if OS == .WINDOWS {
    BeginMenuBar :: () -> bool #foreign imgui "?BeginMenuBar@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    BeginPopup :: (str_id: *u8, flags: WindowFlags = .None) -> bool #foreign imgui "_ZN5ImGui10BeginPopupEPKci";
}

#if OS == .WINDOWS {
    BeginPopup :: (str_id: *u8, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginPopup@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    BeginPopupContextItem :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "_ZN5ImGui21BeginPopupContextItemEPKci";
}

#if OS == .WINDOWS {
    BeginPopupContextItem :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "?BeginPopupContextItem@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    BeginPopupContextVoid :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "_ZN5ImGui21BeginPopupContextVoidEPKci";
}

#if OS == .WINDOWS {
    BeginPopupContextVoid :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "?BeginPopupContextVoid@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    BeginPopupContextWindow :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "_ZN5ImGui23BeginPopupContextWindowEPKci";
}

#if OS == .WINDOWS {
    BeginPopupContextWindow :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> bool #foreign imgui "?BeginPopupContextWindow@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    BeginPopupModal :: (name: *u8, p_open: *bool = null, flags: WindowFlags = .None) -> bool #foreign imgui "_ZN5ImGui15BeginPopupModalEPKcPbi";
}

#if OS == .WINDOWS {
    BeginPopupModal :: (name: *u8, p_open: *bool = null, flags: WindowFlags = .None) -> bool #foreign imgui "?BeginPopupModal@ImGui@@YA_NPEBDPEA_NH@Z";
}

#if OS != .WINDOWS {
    BeginTabBar :: (str_id: *u8, flags: TabBarFlags = .None) -> bool #foreign imgui "_ZN5ImGui11BeginTabBarEPKci";
}

#if OS == .WINDOWS {
    BeginTabBar :: (str_id: *u8, flags: TabBarFlags = .None) -> bool #foreign imgui "?BeginTabBar@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    BeginTabItem :: (label: *u8, p_open: *bool = null, flags: TabItemFlags = .None) -> bool #foreign imgui "_ZN5ImGui12BeginTabItemEPKcPbi";
}

#if OS == .WINDOWS {
    BeginTabItem :: (label: *u8, p_open: *bool = null, flags: TabItemFlags = .None) -> bool #foreign imgui "?BeginTabItem@ImGui@@YA_NPEBDPEA_NH@Z";
}

#if OS != .WINDOWS {
    BeginTable :: (str_id: *u8, column: s32, flags: TableFlags = .None, outer_size: *ImVec2, inner_width: float = 0.0) -> bool #foreign imgui "_ZN5ImGui10BeginTableEPKciiRK6ImVec2f";
}

#if OS == .WINDOWS {
    BeginTable :: (str_id: *u8, column: s32, flags: TableFlags = .None, outer_size: *ImVec2, inner_width: float = 0.0) -> bool #foreign imgui "?BeginTable@ImGui@@YA_NPEBDHHAEBUImVec2@@M@Z";
}

#if OS != .WINDOWS {
    BeginTooltip :: () -> bool #foreign imgui "_ZN5ImGui12BeginTooltipEv";
}

#if OS == .WINDOWS {
    BeginTooltip :: () -> bool #foreign imgui "?BeginTooltip@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    Bullet :: () -> void #foreign imgui "_ZN5ImGui6BulletEv";
}

#if OS == .WINDOWS {
    Bullet :: () -> void #foreign imgui "?Bullet@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    BulletText_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui10BulletTextEPKcz";
}

#if OS == .WINDOWS {
    BulletText_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?BulletText@ImGui@@YAXPEBDZZ";
}

#if OS != .WINDOWS {
    Button :: (label: *u8, size: *ImVec2) -> bool #foreign imgui "_ZN5ImGui6ButtonEPKcRK6ImVec2";
}

#if OS == .WINDOWS {
    Button :: (label: *u8, size: *ImVec2) -> bool #foreign imgui "?Button@ImGui@@YA_NPEBDAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    CalcItemWidth :: () -> float #foreign imgui "_ZN5ImGui13CalcItemWidthEv";
}

#if OS == .WINDOWS {
    CalcItemWidth :: () -> float #foreign imgui "?CalcItemWidth@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    CalcListClipping :: (items_count: s32, items_height: float, out_items_display_start: *s32, out_items_display_end: *s32) -> void #foreign imgui "_ZN5ImGui16CalcListClippingEifPiS0_";
}

#if OS == .WINDOWS {
    CalcListClipping :: (items_count: s32, items_height: float, out_items_display_start: *s32, out_items_display_end: *s32) -> void #foreign imgui "?CalcListClipping@ImGui@@YAXHMPEAH0@Z";
}

#if OS != .WINDOWS {
    CalcTextSize :: (text: *u8, text_end: *u8 = null, hide_text_after_double_hash := false, wrap_width: float = -1.0) -> ImVec2 #foreign imgui "_ZN5ImGui12CalcTextSizeEPKcS1_bf";
}

#if OS == .WINDOWS {
    CalcTextSize :: (text: *u8, text_end: *u8 = null, hide_text_after_double_hash := false, wrap_width: float = -1.0) -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?CalcTextSize@ImGui@@YA?AUImVec2@@PEBD0_NM@Z";
}

#if OS != .WINDOWS {
    Checkbox :: (label: *u8, v: *bool) -> bool #foreign imgui "_ZN5ImGui8CheckboxEPKcPb";
}

#if OS == .WINDOWS {
    Checkbox :: (label: *u8, v: *bool) -> bool #foreign imgui "?Checkbox@ImGui@@YA_NPEBDPEA_N@Z";
}

#if OS != .WINDOWS {
    CheckboxFlags :: (label: *u8, flags: *s32, flags_value: s32) -> bool #foreign imgui "_ZN5ImGui13CheckboxFlagsEPKcPii";
}

#if OS != .WINDOWS {
    CheckboxFlags :: (label: *u8, flags: *u32, flags_value: u32) -> bool #foreign imgui "_ZN5ImGui13CheckboxFlagsEPKcPjj";
}

#if OS == .WINDOWS {
    CheckboxFlags :: (label: *u8, flags: *s32, flags_value: s32) -> bool #foreign imgui "?CheckboxFlags@ImGui@@YA_NPEBDPEAHH@Z";
}

#if OS == .WINDOWS {
    CheckboxFlags :: (label: *u8, flags: *u32, flags_value: u32) -> bool #foreign imgui "?CheckboxFlags@ImGui@@YA_NPEBDPEAII@Z";
}

#if OS != .WINDOWS {
    CloseCurrentPopup :: () -> void #foreign imgui "_ZN5ImGui17CloseCurrentPopupEv";
}

#if OS == .WINDOWS {
    CloseCurrentPopup :: () -> void #foreign imgui "?CloseCurrentPopup@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    CollapsingHeader :: (label: *u8, flags: TreeNodeFlags = .None) -> bool #foreign imgui "_ZN5ImGui16CollapsingHeaderEPKci";
}

#if OS != .WINDOWS {
    CollapsingHeader :: (label: *u8, p_visible: *bool, flags: TreeNodeFlags = .None) -> bool #foreign imgui "_ZN5ImGui16CollapsingHeaderEPKcPbi";
}

#if OS == .WINDOWS {
    CollapsingHeader :: (label: *u8, flags: TreeNodeFlags = .None) -> bool #foreign imgui "?CollapsingHeader@ImGui@@YA_NPEBDH@Z";
}

#if OS == .WINDOWS {
    CollapsingHeader :: (label: *u8, p_visible: *bool, flags: TreeNodeFlags = .None) -> bool #foreign imgui "?CollapsingHeader@ImGui@@YA_NPEBDPEA_NH@Z";
}

#if OS != .WINDOWS {
    ColorButton :: (desc_id: *u8, col: *ImVec4, flags: ColorEditFlags = .None, size: *ImVec2) -> bool #foreign imgui "_ZN5ImGui11ColorButtonEPKcRK6ImVec4iRK6ImVec2";
}

#if OS == .WINDOWS {
    ColorButton :: (desc_id: *u8, col: *ImVec4, flags: ColorEditFlags = .None, size: *ImVec2) -> bool #foreign imgui "?ColorButton@ImGui@@YA_NPEBDAEBUImVec4@@HAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    ColorConvertFloat4ToU32 :: (in: *ImVec4) -> ImU32 #foreign imgui "_ZN5ImGui23ColorConvertFloat4ToU32ERK6ImVec4";
}

#if OS == .WINDOWS {
    ColorConvertFloat4ToU32 :: (in: *ImVec4) -> ImU32 #foreign imgui "?ColorConvertFloat4ToU32@ImGui@@YAIAEBUImVec4@@@Z";
}

#if OS != .WINDOWS {
    ColorConvertHSVtoRGB :: (h: float, s: float, v: float, out_r: *float, out_g: *float, out_b: *float) -> void #foreign imgui "_ZN5ImGui20ColorConvertHSVtoRGBEfffRfS0_S0_";
}

#if OS == .WINDOWS {
    ColorConvertHSVtoRGB :: (h: float, s: float, v: float, out_r: *float, out_g: *float, out_b: *float) -> void #foreign imgui "?ColorConvertHSVtoRGB@ImGui@@YAXMMMAEAM00@Z";
}

#if OS != .WINDOWS {
    ColorConvertRGBtoHSV :: (r: float, g: float, b: float, out_h: *float, out_s: *float, out_v: *float) -> void #foreign imgui "_ZN5ImGui20ColorConvertRGBtoHSVEfffRfS0_S0_";
}

#if OS == .WINDOWS {
    ColorConvertRGBtoHSV :: (r: float, g: float, b: float, out_h: *float, out_s: *float, out_v: *float) -> void #foreign imgui "?ColorConvertRGBtoHSV@ImGui@@YAXMMMAEAM00@Z";
}

#if OS != .WINDOWS {
    ColorConvertU32ToFloat4 :: (in: ImU32) -> ImVec4 #foreign imgui "_ZN5ImGui23ColorConvertU32ToFloat4Ej";
}

#if OS == .WINDOWS {
    ColorConvertU32ToFloat4 :: (in: ImU32) -> ImVec4 #cpp_return_type_is_non_pod #foreign imgui "?ColorConvertU32ToFloat4@ImGui@@YA?AUImVec4@@I@Z";
}

#if OS != .WINDOWS {
    ColorEdit3 :: (label: *u8, col: *[3] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "_ZN5ImGui10ColorEdit3EPKcPfi";
}

#if OS == .WINDOWS {
    ColorEdit3 :: (label: *u8, col: *[3] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "?ColorEdit3@ImGui@@YA_NPEBDQEAMH@Z";
}

#if OS != .WINDOWS {
    ColorEdit4 :: (label: *u8, col: *[4] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "_ZN5ImGui10ColorEdit4EPKcPfi";
}

#if OS == .WINDOWS {
    ColorEdit4 :: (label: *u8, col: *[4] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "?ColorEdit4@ImGui@@YA_NPEBDQEAMH@Z";
}

#if OS != .WINDOWS {
    ColorPicker3 :: (label: *u8, col: *[3] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "_ZN5ImGui12ColorPicker3EPKcPfi";
}

#if OS == .WINDOWS {
    ColorPicker3 :: (label: *u8, col: *[3] float, flags: ColorEditFlags = .None) -> bool #foreign imgui "?ColorPicker3@ImGui@@YA_NPEBDQEAMH@Z";
}

#if OS != .WINDOWS {
    ColorPicker4 :: (label: *u8, col: *[4] float, flags: ColorEditFlags = .None, ref_col: *float = null) -> bool #foreign imgui "_ZN5ImGui12ColorPicker4EPKcPfiPKf";
}

#if OS == .WINDOWS {
    ColorPicker4 :: (label: *u8, col: *[4] float, flags: ColorEditFlags = .None, ref_col: *float = null) -> bool #foreign imgui "?ColorPicker4@ImGui@@YA_NPEBDQEAMHPEBM@Z";
}

#if OS != .WINDOWS {
    Columns :: (count: s32 = 1, id: *u8 = null, border := true) -> void #foreign imgui "_ZN5ImGui7ColumnsEiPKcb";
}

#if OS == .WINDOWS {
    Columns :: (count: s32 = 1, id: *u8 = null, border := true) -> void #foreign imgui "?Columns@ImGui@@YAXHPEBD_N@Z";
}

#if OS != .WINDOWS {
    Combo :: (label: *u8, current_item: *s32, items: **u8, items_count: s32, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "_ZN5ImGui5ComboEPKcPiPKS1_ii";
}

#if OS != .WINDOWS {
    Combo :: (label: *u8, current_item: *s32, items_separated_by_zeros: *u8, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "_ZN5ImGui5ComboEPKcPiS1_i";
}

#if OS != .WINDOWS {
    Combo :: (label: *u8, current_item: *s32, items_getter: #type (data: *void, idx: s32, out_text: **u8) -> bool #c_call, data: *void, items_count: s32, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "_ZN5ImGui5ComboEPKcPiPFbPviPS1_ES3_ii";
}

#if OS == .WINDOWS {
    Combo :: (label: *u8, current_item: *s32, items: **u8, items_count: s32, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "?Combo@ImGui@@YA_NPEBDPEAHQEBQEBDHH@Z";
}

#if OS == .WINDOWS {
    Combo :: (label: *u8, current_item: *s32, items_separated_by_zeros: *u8, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "?Combo@ImGui@@YA_NPEBDPEAH0H@Z";
}

#if OS == .WINDOWS {
    Combo :: (label: *u8, current_item: *s32, items_getter: #type (data: *void, idx: s32, out_text: **u8) -> bool #c_call, data: *void, items_count: s32, popup_max_height_in_items: s32 = -1) -> bool #foreign imgui "?Combo@ImGui@@YA_NPEBDPEAHP6A_NPEAXHPEAPEBD@Z2HH@Z";
}

#if OS != .WINDOWS {
    CreateContext :: (shared_font_atlas: *ImFontAtlas = null) -> *ImGuiContext #foreign imgui "_ZN5ImGui13CreateContextEP11ImFontAtlas";
}

#if OS == .WINDOWS {
    CreateContext :: (shared_font_atlas: *ImFontAtlas = null) -> *ImGuiContext #foreign imgui "?CreateContext@ImGui@@YAPEAUImGuiContext@@PEAUImFontAtlas@@@Z";
}

#if OS != .WINDOWS {
    DebugCheckVersionAndDataLayout :: (version_str: *u8, sz_io: u64, sz_style: u64, sz_vec2: u64, sz_vec4: u64, sz_drawvert: u64, sz_drawidx: u64) -> bool #foreign imgui "_ZN5ImGui30DebugCheckVersionAndDataLayoutEPKcmmmmmm";
}

#if OS == .WINDOWS {
    DebugCheckVersionAndDataLayout :: (version_str: *u8, sz_io: u64, sz_style: u64, sz_vec2: u64, sz_vec4: u64, sz_drawvert: u64, sz_drawidx: u64) -> bool #foreign imgui "?DebugCheckVersionAndDataLayout@ImGui@@YA_NPEBD_K11111@Z";
}

#if OS != .WINDOWS {
    DebugTextEncoding :: (text: *u8) -> void #foreign imgui "_ZN5ImGui17DebugTextEncodingEPKc";
}

#if OS == .WINDOWS {
    DebugTextEncoding :: (text: *u8) -> void #foreign imgui "?DebugTextEncoding@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    DestroyContext :: (ctx: *ImGuiContext = null) -> void #foreign imgui "_ZN5ImGui14DestroyContextEP12ImGuiContext";
}

#if OS == .WINDOWS {
    DestroyContext :: (ctx: *ImGuiContext = null) -> void #foreign imgui "?DestroyContext@ImGui@@YAXPEAUImGuiContext@@@Z";
}

#if OS != .WINDOWS {
    DragFloat :: (label: *u8, v: *float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui9DragFloatEPKcPffffS1_i";
}

#if OS == .WINDOWS {
    DragFloat :: (label: *u8, v: *float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat@ImGui@@YA_NPEBDPEAMMMM0H@Z";
}

#if OS != .WINDOWS {
    DragFloat2 :: (label: *u8, v: *[2] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10DragFloat2EPKcPffffS1_i";
}

#if OS == .WINDOWS {
    DragFloat2 :: (label: *u8, v: *[2] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat2@ImGui@@YA_NPEBDQEAMMMM0H@Z";
}

#if OS != .WINDOWS {
    DragFloat3 :: (label: *u8, v: *[3] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10DragFloat3EPKcPffffS1_i";
}

#if OS == .WINDOWS {
    DragFloat3 :: (label: *u8, v: *[3] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat3@ImGui@@YA_NPEBDQEAMMMM0H@Z";
}

#if OS != .WINDOWS {
    DragFloat4 :: (label: *u8, v: *[4] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10DragFloat4EPKcPffffS1_i";
}

#if OS == .WINDOWS {
    DragFloat4 :: (label: *u8, v: *[4] float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloat4@ImGui@@YA_NPEBDQEAMMMM0H@Z";
}

#if OS != .WINDOWS {
    DragFloatRange2 :: (label: *u8, v_current_min: *float, v_current_max: *float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", format_max: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui15DragFloatRange2EPKcPfS2_fffS1_S1_i";
}

#if OS == .WINDOWS {
    DragFloatRange2 :: (label: *u8, v_current_min: *float, v_current_max: *float, v_speed: float = 1.0, v_min: float = 0.0, v_max: float = 0.0, format: *u8 = "%.3f", format_max: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragFloatRange2@ImGui@@YA_NPEBDPEAM1MMM00H@Z";
}

#if OS != .WINDOWS {
    DragInt :: (label: *u8, v: *s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui7DragIntEPKcPifiiS1_i";
}

#if OS == .WINDOWS {
    DragInt :: (label: *u8, v: *s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt@ImGui@@YA_NPEBDPEAHMHH0H@Z";
}

#if OS != .WINDOWS {
    DragInt2 :: (label: *u8, v: *[2] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui8DragInt2EPKcPifiiS1_i";
}

#if OS == .WINDOWS {
    DragInt2 :: (label: *u8, v: *[2] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt2@ImGui@@YA_NPEBDQEAHMHH0H@Z";
}

#if OS != .WINDOWS {
    DragInt3 :: (label: *u8, v: *[3] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui8DragInt3EPKcPifiiS1_i";
}

#if OS == .WINDOWS {
    DragInt3 :: (label: *u8, v: *[3] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt3@ImGui@@YA_NPEBDQEAHMHH0H@Z";
}

#if OS != .WINDOWS {
    DragInt4 :: (label: *u8, v: *[4] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui8DragInt4EPKcPifiiS1_i";
}

#if OS == .WINDOWS {
    DragInt4 :: (label: *u8, v: *[4] s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?DragInt4@ImGui@@YA_NPEBDQEAHMHH0H@Z";
}

#if OS != .WINDOWS {
    DragIntRange2 :: (label: *u8, v_current_min: *s32, v_current_max: *s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", format_max: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui13DragIntRange2EPKcPiS2_fiiS1_S1_i";
}

#if OS == .WINDOWS {
    DragIntRange2 :: (label: *u8, v_current_min: *s32, v_current_max: *s32, v_speed: float = 1.0, v_min: s32 = 0, v_max: s32 = 0, format: *u8 = "%d", format_max: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragIntRange2@ImGui@@YA_NPEBDPEAH1MHH00H@Z";
}

#if OS != .WINDOWS {
    DragScalar :: (label: *u8, data_type: DataType, p_data: *void, v_speed: float = 1.0, p_min: *void = null, p_max: *void = null, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10DragScalarEPKciPvfPKvS4_S1_i";
}

#if OS == .WINDOWS {
    DragScalar :: (label: *u8, data_type: DataType, p_data: *void, v_speed: float = 1.0, p_min: *void = null, p_max: *void = null, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragScalar@ImGui@@YA_NPEBDHPEAXMPEBX20H@Z";
}

#if OS != .WINDOWS {
    DragScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, v_speed: float = 1.0, p_min: *void = null, p_max: *void = null, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui11DragScalarNEPKciPvifPKvS4_S1_i";
}

#if OS == .WINDOWS {
    DragScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, v_speed: float = 1.0, p_min: *void = null, p_max: *void = null, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?DragScalarN@ImGui@@YA_NPEBDHPEAXHMPEBX20H@Z";
}

#if OS != .WINDOWS {
    Dummy :: (size: *ImVec2) -> void #foreign imgui "_ZN5ImGui5DummyERK6ImVec2";
}

#if OS == .WINDOWS {
    Dummy :: (size: *ImVec2) -> void #foreign imgui "?Dummy@ImGui@@YAXAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    End :: () -> void #foreign imgui "_ZN5ImGui3EndEv";
}

#if OS == .WINDOWS {
    End :: () -> void #foreign imgui "?End@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndChild :: () -> void #foreign imgui "_ZN5ImGui8EndChildEv";
}

#if OS == .WINDOWS {
    EndChild :: () -> void #foreign imgui "?EndChild@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndChildFrame :: () -> void #foreign imgui "_ZN5ImGui13EndChildFrameEv";
}

#if OS == .WINDOWS {
    EndChildFrame :: () -> void #foreign imgui "?EndChildFrame@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndCombo :: () -> void #foreign imgui "_ZN5ImGui8EndComboEv";
}

#if OS == .WINDOWS {
    EndCombo :: () -> void #foreign imgui "?EndCombo@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndDisabled :: () -> void #foreign imgui "_ZN5ImGui11EndDisabledEv";
}

#if OS == .WINDOWS {
    EndDisabled :: () -> void #foreign imgui "?EndDisabled@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndDragDropSource :: () -> void #foreign imgui "_ZN5ImGui17EndDragDropSourceEv";
}

#if OS == .WINDOWS {
    EndDragDropSource :: () -> void #foreign imgui "?EndDragDropSource@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndDragDropTarget :: () -> void #foreign imgui "_ZN5ImGui17EndDragDropTargetEv";
}

#if OS == .WINDOWS {
    EndDragDropTarget :: () -> void #foreign imgui "?EndDragDropTarget@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndFrame :: () -> void #foreign imgui "_ZN5ImGui8EndFrameEv";
}

#if OS == .WINDOWS {
    EndFrame :: () -> void #foreign imgui "?EndFrame@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndGroup :: () -> void #foreign imgui "_ZN5ImGui8EndGroupEv";
}

#if OS == .WINDOWS {
    EndGroup :: () -> void #foreign imgui "?EndGroup@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndListBox :: () -> void #foreign imgui "_ZN5ImGui10EndListBoxEv";
}

#if OS == .WINDOWS {
    EndListBox :: () -> void #foreign imgui "?EndListBox@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndMainMenuBar :: () -> void #foreign imgui "_ZN5ImGui14EndMainMenuBarEv";
}

#if OS == .WINDOWS {
    EndMainMenuBar :: () -> void #foreign imgui "?EndMainMenuBar@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndMenu :: () -> void #foreign imgui "_ZN5ImGui7EndMenuEv";
}

#if OS == .WINDOWS {
    EndMenu :: () -> void #foreign imgui "?EndMenu@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndMenuBar :: () -> void #foreign imgui "_ZN5ImGui10EndMenuBarEv";
}

#if OS == .WINDOWS {
    EndMenuBar :: () -> void #foreign imgui "?EndMenuBar@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndPopup :: () -> void #foreign imgui "_ZN5ImGui8EndPopupEv";
}

#if OS == .WINDOWS {
    EndPopup :: () -> void #foreign imgui "?EndPopup@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndTabBar :: () -> void #foreign imgui "_ZN5ImGui9EndTabBarEv";
}

#if OS == .WINDOWS {
    EndTabBar :: () -> void #foreign imgui "?EndTabBar@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndTabItem :: () -> void #foreign imgui "_ZN5ImGui10EndTabItemEv";
}

#if OS == .WINDOWS {
    EndTabItem :: () -> void #foreign imgui "?EndTabItem@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndTable :: () -> void #foreign imgui "_ZN5ImGui8EndTableEv";
}

#if OS == .WINDOWS {
    EndTable :: () -> void #foreign imgui "?EndTable@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    EndTooltip :: () -> void #foreign imgui "_ZN5ImGui10EndTooltipEv";
}

#if OS == .WINDOWS {
    EndTooltip :: () -> void #foreign imgui "?EndTooltip@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    GetAllocatorFunctions :: (p_alloc_func: *MemAllocFunc, p_free_func: *MemFreeFunc, p_user_data: **void) -> void #foreign imgui "_ZN5ImGui21GetAllocatorFunctionsEPPFPvmS0_EPPFvS0_S0_EPS0_";
}

#if OS == .WINDOWS {
    GetAllocatorFunctions :: (p_alloc_func: *MemAllocFunc, p_free_func: *MemFreeFunc, p_user_data: **void) -> void #foreign imgui "?GetAllocatorFunctions@ImGui@@YAXPEAP6APEAX_KPEAX@ZPEAP6AX11@ZPEAPEAX@Z";
}

#if OS != .WINDOWS {
    GetBackgroundDrawList :: () -> *ImDrawList #foreign imgui "_ZN5ImGui21GetBackgroundDrawListEv";
}

#if OS == .WINDOWS {
    GetBackgroundDrawList :: () -> *ImDrawList #foreign imgui "?GetBackgroundDrawList@ImGui@@YAPEAUImDrawList@@XZ";
}

#if OS != .WINDOWS {
    GetClipboardText :: () -> *u8 #foreign imgui "_ZN5ImGui16GetClipboardTextEv";
}

#if OS == .WINDOWS {
    GetClipboardText :: () -> *u8 #foreign imgui "?GetClipboardText@ImGui@@YAPEBDXZ";
}

#if OS != .WINDOWS {
    GetColorU32 :: (idx: Col, alpha_mul: float = 1.0) -> ImU32 #foreign imgui "_ZN5ImGui11GetColorU32Eif";
}

#if OS != .WINDOWS {
    GetColorU32 :: (col: *ImVec4) -> ImU32 #foreign imgui "_ZN5ImGui11GetColorU32ERK6ImVec4";
}

#if OS != .WINDOWS {
    GetColorU32 :: (col: ImU32) -> ImU32 #foreign imgui "_ZN5ImGui11GetColorU32Ej";
}

#if OS == .WINDOWS {
    GetColorU32 :: (idx: Col, alpha_mul: float = 1.0) -> ImU32 #foreign imgui "?GetColorU32@ImGui@@YAIHM@Z";
}

#if OS == .WINDOWS {
    GetColorU32 :: (col: *ImVec4) -> ImU32 #foreign imgui "?GetColorU32@ImGui@@YAIAEBUImVec4@@@Z";
}

#if OS == .WINDOWS {
    GetColorU32 :: (col: ImU32) -> ImU32 #foreign imgui "?GetColorU32@ImGui@@YAII@Z";
}

#if OS != .WINDOWS {
    GetColumnIndex :: () -> s32 #foreign imgui "_ZN5ImGui14GetColumnIndexEv";
}

#if OS == .WINDOWS {
    GetColumnIndex :: () -> s32 #foreign imgui "?GetColumnIndex@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    GetColumnOffset :: (column_index: s32 = -1) -> float #foreign imgui "_ZN5ImGui15GetColumnOffsetEi";
}

#if OS == .WINDOWS {
    GetColumnOffset :: (column_index: s32 = -1) -> float #foreign imgui "?GetColumnOffset@ImGui@@YAMH@Z";
}

#if OS != .WINDOWS {
    GetColumnsCount :: () -> s32 #foreign imgui "_ZN5ImGui15GetColumnsCountEv";
}

#if OS == .WINDOWS {
    GetColumnsCount :: () -> s32 #foreign imgui "?GetColumnsCount@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    GetColumnWidth :: (column_index: s32 = -1) -> float #foreign imgui "_ZN5ImGui14GetColumnWidthEi";
}

#if OS == .WINDOWS {
    GetColumnWidth :: (column_index: s32 = -1) -> float #foreign imgui "?GetColumnWidth@ImGui@@YAMH@Z";
}

#if OS != .WINDOWS {
    GetContentRegionAvail :: () -> ImVec2 #foreign imgui "_ZN5ImGui21GetContentRegionAvailEv";
}

#if OS == .WINDOWS {
    GetContentRegionAvail :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetContentRegionAvail@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetContentRegionMax :: () -> ImVec2 #foreign imgui "_ZN5ImGui19GetContentRegionMaxEv";
}

#if OS == .WINDOWS {
    GetContentRegionMax :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetContentRegionMax@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetCurrentContext :: () -> *ImGuiContext #foreign imgui "_ZN5ImGui17GetCurrentContextEv";
}

#if OS == .WINDOWS {
    GetCurrentContext :: () -> *ImGuiContext #foreign imgui "?GetCurrentContext@ImGui@@YAPEAUImGuiContext@@XZ";
}

#if OS != .WINDOWS {
    GetCursorPos :: () -> ImVec2 #foreign imgui "_ZN5ImGui12GetCursorPosEv";
}

#if OS == .WINDOWS {
    GetCursorPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetCursorPos@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetCursorPosX :: () -> float #foreign imgui "_ZN5ImGui13GetCursorPosXEv";
}

#if OS == .WINDOWS {
    GetCursorPosX :: () -> float #foreign imgui "?GetCursorPosX@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetCursorPosY :: () -> float #foreign imgui "_ZN5ImGui13GetCursorPosYEv";
}

#if OS == .WINDOWS {
    GetCursorPosY :: () -> float #foreign imgui "?GetCursorPosY@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetCursorScreenPos :: () -> ImVec2 #foreign imgui "_ZN5ImGui18GetCursorScreenPosEv";
}

#if OS == .WINDOWS {
    GetCursorScreenPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetCursorScreenPos@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetCursorStartPos :: () -> ImVec2 #foreign imgui "_ZN5ImGui17GetCursorStartPosEv";
}

#if OS == .WINDOWS {
    GetCursorStartPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetCursorStartPos@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetDragDropPayload :: () -> *Payload #foreign imgui "_ZN5ImGui18GetDragDropPayloadEv";
}

#if OS == .WINDOWS {
    GetDragDropPayload :: () -> *Payload #foreign imgui "?GetDragDropPayload@ImGui@@YAPEBUImGuiPayload@@XZ";
}

#if OS != .WINDOWS {
    GetDrawData :: () -> *ImDrawData #foreign imgui "_ZN5ImGui11GetDrawDataEv";
}

#if OS == .WINDOWS {
    GetDrawData :: () -> *ImDrawData #foreign imgui "?GetDrawData@ImGui@@YAPEAUImDrawData@@XZ";
}

#if OS != .WINDOWS {
    GetDrawListSharedData :: () -> *ImDrawListSharedData #foreign imgui "_ZN5ImGui21GetDrawListSharedDataEv";
}

#if OS == .WINDOWS {
    GetDrawListSharedData :: () -> *ImDrawListSharedData #foreign imgui "?GetDrawListSharedData@ImGui@@YAPEAUImDrawListSharedData@@XZ";
}

#if OS != .WINDOWS {
    GetFont :: () -> *ImFont #foreign imgui "_ZN5ImGui7GetFontEv";
}

#if OS == .WINDOWS {
    GetFont :: () -> *ImFont #foreign imgui "?GetFont@ImGui@@YAPEAUImFont@@XZ";
}

#if OS != .WINDOWS {
    GetFontSize :: () -> float #foreign imgui "_ZN5ImGui11GetFontSizeEv";
}

#if OS == .WINDOWS {
    GetFontSize :: () -> float #foreign imgui "?GetFontSize@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetFontTexUvWhitePixel :: () -> ImVec2 #foreign imgui "_ZN5ImGui22GetFontTexUvWhitePixelEv";
}

#if OS == .WINDOWS {
    GetFontTexUvWhitePixel :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetFontTexUvWhitePixel@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetForegroundDrawList :: () -> *ImDrawList #foreign imgui "_ZN5ImGui21GetForegroundDrawListEv";
}

#if OS == .WINDOWS {
    GetForegroundDrawList :: () -> *ImDrawList #foreign imgui "?GetForegroundDrawList@ImGui@@YAPEAUImDrawList@@XZ";
}

#if OS != .WINDOWS {
    GetFrameCount :: () -> s32 #foreign imgui "_ZN5ImGui13GetFrameCountEv";
}

#if OS == .WINDOWS {
    GetFrameCount :: () -> s32 #foreign imgui "?GetFrameCount@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    GetFrameHeight :: () -> float #foreign imgui "_ZN5ImGui14GetFrameHeightEv";
}

#if OS == .WINDOWS {
    GetFrameHeight :: () -> float #foreign imgui "?GetFrameHeight@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetFrameHeightWithSpacing :: () -> float #foreign imgui "_ZN5ImGui25GetFrameHeightWithSpacingEv";
}

#if OS == .WINDOWS {
    GetFrameHeightWithSpacing :: () -> float #foreign imgui "?GetFrameHeightWithSpacing@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetID :: (str_id: *u8) -> ID #foreign imgui "_ZN5ImGui5GetIDEPKc";
}

#if OS != .WINDOWS {
    GetID :: (str_id_begin: *u8, str_id_end: *u8) -> ID #foreign imgui "_ZN5ImGui5GetIDEPKcS1_";
}

#if OS != .WINDOWS {
    GetID :: (ptr_id: *void) -> ID #foreign imgui "_ZN5ImGui5GetIDEPKv";
}

#if OS == .WINDOWS {
    GetID :: (str_id: *u8) -> ID #foreign imgui "?GetID@ImGui@@YAIPEBD@Z";
}

#if OS == .WINDOWS {
    GetID :: (str_id_begin: *u8, str_id_end: *u8) -> ID #foreign imgui "?GetID@ImGui@@YAIPEBD0@Z";
}

#if OS == .WINDOWS {
    GetID :: (ptr_id: *void) -> ID #foreign imgui "?GetID@ImGui@@YAIPEBX@Z";
}

#if OS != .WINDOWS {
    GetIO :: () -> *IO #foreign imgui "_ZN5ImGui5GetIOEv";
}

#if OS == .WINDOWS {
    GetIO :: () -> *IO #foreign imgui "?GetIO@ImGui@@YAAEAUImGuiIO@@XZ";
}

#if OS != .WINDOWS {
    GetItemID :: () -> ID #foreign imgui "_ZN5ImGui9GetItemIDEv";
}

#if OS == .WINDOWS {
    GetItemID :: () -> ID #foreign imgui "?GetItemID@ImGui@@YAIXZ";
}

#if OS != .WINDOWS {
    GetItemRectMax :: () -> ImVec2 #foreign imgui "_ZN5ImGui14GetItemRectMaxEv";
}

#if OS == .WINDOWS {
    GetItemRectMax :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetItemRectMax@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetItemRectMin :: () -> ImVec2 #foreign imgui "_ZN5ImGui14GetItemRectMinEv";
}

#if OS == .WINDOWS {
    GetItemRectMin :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetItemRectMin@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetItemRectSize :: () -> ImVec2 #foreign imgui "_ZN5ImGui15GetItemRectSizeEv";
}

#if OS == .WINDOWS {
    GetItemRectSize :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetItemRectSize@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetKeyIndex :: (key: Key) -> Key #foreign imgui "_ZN5ImGui11GetKeyIndexE8ImGuiKey";
}

#if OS == .WINDOWS {
    GetKeyIndex :: (key: Key) -> Key #foreign imgui "?GetKeyIndex@ImGui@@YA?AW4ImGuiKey@@W42@@Z";
}

#if OS != .WINDOWS {
    GetKeyName :: (key: Key) -> *u8 #foreign imgui "_ZN5ImGui10GetKeyNameE8ImGuiKey";
}

#if OS == .WINDOWS {
    GetKeyName :: (key: Key) -> *u8 #foreign imgui "?GetKeyName@ImGui@@YAPEBDW4ImGuiKey@@@Z";
}

#if OS != .WINDOWS {
    GetKeyPressedAmount :: (key: Key, repeat_delay: float, rate: float) -> s32 #foreign imgui "_ZN5ImGui19GetKeyPressedAmountE8ImGuiKeyff";
}

#if OS == .WINDOWS {
    GetKeyPressedAmount :: (key: Key, repeat_delay: float, rate: float) -> s32 #foreign imgui "?GetKeyPressedAmount@ImGui@@YAHW4ImGuiKey@@MM@Z";
}

#if OS != .WINDOWS {
    GetMainViewport :: () -> *Viewport #foreign imgui "_ZN5ImGui15GetMainViewportEv";
}

#if OS == .WINDOWS {
    GetMainViewport :: () -> *Viewport #foreign imgui "?GetMainViewport@ImGui@@YAPEAUImGuiViewport@@XZ";
}

#if OS != .WINDOWS {
    GetMouseClickedCount :: (button: MouseButton) -> s32 #foreign imgui "_ZN5ImGui20GetMouseClickedCountEi";
}

#if OS == .WINDOWS {
    GetMouseClickedCount :: (button: MouseButton) -> s32 #foreign imgui "?GetMouseClickedCount@ImGui@@YAHH@Z";
}

#if OS != .WINDOWS {
    GetMouseCursor :: () -> MouseCursor #foreign imgui "_ZN5ImGui14GetMouseCursorEv";
}

#if OS == .WINDOWS {
    GetMouseCursor :: () -> MouseCursor #foreign imgui "?GetMouseCursor@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    GetMouseDragDelta :: (button: MouseButton = .Left, lock_threshold: float = -1.0) -> ImVec2 #foreign imgui "_ZN5ImGui17GetMouseDragDeltaEif";
}

#if OS == .WINDOWS {
    GetMouseDragDelta :: (button: MouseButton = .Left, lock_threshold: float = -1.0) -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetMouseDragDelta@ImGui@@YA?AUImVec2@@HM@Z";
}

#if OS != .WINDOWS {
    GetMousePos :: () -> ImVec2 #foreign imgui "_ZN5ImGui11GetMousePosEv";
}

#if OS == .WINDOWS {
    GetMousePos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetMousePos@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetMousePosOnOpeningCurrentPopup :: () -> ImVec2 #foreign imgui "_ZN5ImGui32GetMousePosOnOpeningCurrentPopupEv";
}

#if OS == .WINDOWS {
    GetMousePosOnOpeningCurrentPopup :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetMousePosOnOpeningCurrentPopup@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetScrollMaxX :: () -> float #foreign imgui "_ZN5ImGui13GetScrollMaxXEv";
}

#if OS == .WINDOWS {
    GetScrollMaxX :: () -> float #foreign imgui "?GetScrollMaxX@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetScrollMaxY :: () -> float #foreign imgui "_ZN5ImGui13GetScrollMaxYEv";
}

#if OS == .WINDOWS {
    GetScrollMaxY :: () -> float #foreign imgui "?GetScrollMaxY@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetScrollX :: () -> float #foreign imgui "_ZN5ImGui10GetScrollXEv";
}

#if OS == .WINDOWS {
    GetScrollX :: () -> float #foreign imgui "?GetScrollX@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetScrollY :: () -> float #foreign imgui "_ZN5ImGui10GetScrollYEv";
}

#if OS == .WINDOWS {
    GetScrollY :: () -> float #foreign imgui "?GetScrollY@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetStateStorage :: () -> *Storage #foreign imgui "_ZN5ImGui15GetStateStorageEv";
}

#if OS == .WINDOWS {
    GetStateStorage :: () -> *Storage #foreign imgui "?GetStateStorage@ImGui@@YAPEAUImGuiStorage@@XZ";
}

#if OS != .WINDOWS {
    GetStyle :: () -> *Style #foreign imgui "_ZN5ImGui8GetStyleEv";
}

#if OS == .WINDOWS {
    GetStyle :: () -> *Style #foreign imgui "?GetStyle@ImGui@@YAAEAUImGuiStyle@@XZ";
}

#if OS != .WINDOWS {
    GetStyleColorName :: (idx: Col) -> *u8 #foreign imgui "_ZN5ImGui17GetStyleColorNameEi";
}

#if OS == .WINDOWS {
    GetStyleColorName :: (idx: Col) -> *u8 #foreign imgui "?GetStyleColorName@ImGui@@YAPEBDH@Z";
}

#if OS != .WINDOWS {
    GetStyleColorVec4 :: (idx: Col) -> *ImVec4 #foreign imgui "_ZN5ImGui17GetStyleColorVec4Ei";
}

#if OS == .WINDOWS {
    GetStyleColorVec4 :: (idx: Col) -> *ImVec4 #foreign imgui "?GetStyleColorVec4@ImGui@@YAAEBUImVec4@@H@Z";
}

#if OS != .WINDOWS {
    GetTextLineHeight :: () -> float #foreign imgui "_ZN5ImGui17GetTextLineHeightEv";
}

#if OS == .WINDOWS {
    GetTextLineHeight :: () -> float #foreign imgui "?GetTextLineHeight@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetTextLineHeightWithSpacing :: () -> float #foreign imgui "_ZN5ImGui28GetTextLineHeightWithSpacingEv";
}

#if OS == .WINDOWS {
    GetTextLineHeightWithSpacing :: () -> float #foreign imgui "?GetTextLineHeightWithSpacing@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetTime :: () -> float64 #foreign imgui "_ZN5ImGui7GetTimeEv";
}

#if OS == .WINDOWS {
    GetTime :: () -> float64 #foreign imgui "?GetTime@ImGui@@YANXZ";
}

#if OS != .WINDOWS {
    GetTreeNodeToLabelSpacing :: () -> float #foreign imgui "_ZN5ImGui25GetTreeNodeToLabelSpacingEv";
}

#if OS == .WINDOWS {
    GetTreeNodeToLabelSpacing :: () -> float #foreign imgui "?GetTreeNodeToLabelSpacing@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetVersion :: () -> *u8 #foreign imgui "_ZN5ImGui10GetVersionEv";
}

#if OS == .WINDOWS {
    GetVersion :: () -> *u8 #foreign imgui "?GetVersion@ImGui@@YAPEBDXZ";
}

#if OS != .WINDOWS {
    GetWindowContentRegionMax :: () -> ImVec2 #foreign imgui "_ZN5ImGui25GetWindowContentRegionMaxEv";
}

#if OS == .WINDOWS {
    GetWindowContentRegionMax :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowContentRegionMax@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetWindowContentRegionMin :: () -> ImVec2 #foreign imgui "_ZN5ImGui25GetWindowContentRegionMinEv";
}

#if OS == .WINDOWS {
    GetWindowContentRegionMin :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowContentRegionMin@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetWindowDrawList :: () -> *ImDrawList #foreign imgui "_ZN5ImGui17GetWindowDrawListEv";
}

#if OS == .WINDOWS {
    GetWindowDrawList :: () -> *ImDrawList #foreign imgui "?GetWindowDrawList@ImGui@@YAPEAUImDrawList@@XZ";
}

#if OS != .WINDOWS {
    GetWindowHeight :: () -> float #foreign imgui "_ZN5ImGui15GetWindowHeightEv";
}

#if OS == .WINDOWS {
    GetWindowHeight :: () -> float #foreign imgui "?GetWindowHeight@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    GetWindowPos :: () -> ImVec2 #foreign imgui "_ZN5ImGui12GetWindowPosEv";
}

#if OS == .WINDOWS {
    GetWindowPos :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowPos@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetWindowSize :: () -> ImVec2 #foreign imgui "_ZN5ImGui13GetWindowSizeEv";
}

#if OS == .WINDOWS {
    GetWindowSize :: () -> ImVec2 #cpp_return_type_is_non_pod #foreign imgui "?GetWindowSize@ImGui@@YA?AUImVec2@@XZ";
}

#if OS != .WINDOWS {
    GetWindowWidth :: () -> float #foreign imgui "_ZN5ImGui14GetWindowWidthEv";
}

#if OS == .WINDOWS {
    GetWindowWidth :: () -> float #foreign imgui "?GetWindowWidth@ImGui@@YAMXZ";
}

#if OS != .WINDOWS {
    Image :: (user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, tint_col: *ImVec4, border_col: *ImVec4) -> void #foreign imgui "_ZN5ImGui5ImageEPvRK6ImVec2S3_S3_RK6ImVec4S6_";
}

#if OS == .WINDOWS {
    Image :: (user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, tint_col: *ImVec4, border_col: *ImVec4) -> void #foreign imgui "?Image@ImGui@@YAXPEAXAEBUImVec2@@11AEBUImVec4@@2@Z";
}

#if OS != .WINDOWS {
    ImageButton :: (str_id: *u8, user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, bg_col: *ImVec4, tint_col: *ImVec4) -> bool #foreign imgui "_ZN5ImGui11ImageButtonEPKcPvRK6ImVec2S5_S5_RK6ImVec4S8_";
}

#if OS != .WINDOWS {
    ImageButton :: (user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, frame_padding: s32 = -1, bg_col: *ImVec4, tint_col: *ImVec4) -> bool #foreign imgui "_ZN5ImGui11ImageButtonEPvRK6ImVec2S3_S3_iRK6ImVec4S6_";
}

#if OS == .WINDOWS {
    ImageButton :: (str_id: *u8, user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, bg_col: *ImVec4, tint_col: *ImVec4) -> bool #foreign imgui "?ImageButton@ImGui@@YA_NPEBDPEAXAEBUImVec2@@22AEBUImVec4@@3@Z";
}

#if OS == .WINDOWS {
    ImageButton :: (user_texture_id: ImTextureID, size: *ImVec2, uv0: *ImVec2, uv1: *ImVec2, frame_padding: s32 = -1, bg_col: *ImVec4, tint_col: *ImVec4) -> bool #foreign imgui "?ImageButton@ImGui@@YA_NPEAXAEBUImVec2@@11HAEBUImVec4@@2@Z";
}

#if OS != .WINDOWS {
    Indent :: (indent_w: float = 0.0) -> void #foreign imgui "_ZN5ImGui6IndentEf";
}

#if OS == .WINDOWS {
    Indent :: (indent_w: float = 0.0) -> void #foreign imgui "?Indent@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    InputDouble :: (label: *u8, v: *float64, step: float64 = 0.0, step_fast: float64 = 0.0, format: *u8 = "%.6f", flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui11InputDoubleEPKcPdddS1_i";
}

#if OS == .WINDOWS {
    InputDouble :: (label: *u8, v: *float64, step: float64 = 0.0, step_fast: float64 = 0.0, format: *u8 = "%.6f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputDouble@ImGui@@YA_NPEBDPEANNN0H@Z";
}

#if OS != .WINDOWS {
    InputFloat :: (label: *u8, v: *float, step: float = 0.0, step_fast: float = 0.0, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui10InputFloatEPKcPfffS1_i";
}

#if OS == .WINDOWS {
    InputFloat :: (label: *u8, v: *float, step: float = 0.0, step_fast: float = 0.0, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat@ImGui@@YA_NPEBDPEAMMM0H@Z";
}

#if OS != .WINDOWS {
    InputFloat2 :: (label: *u8, v: *[2] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui11InputFloat2EPKcPfS1_i";
}

#if OS == .WINDOWS {
    InputFloat2 :: (label: *u8, v: *[2] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat2@ImGui@@YA_NPEBDQEAM0H@Z";
}

#if OS != .WINDOWS {
    InputFloat3 :: (label: *u8, v: *[3] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui11InputFloat3EPKcPfS1_i";
}

#if OS == .WINDOWS {
    InputFloat3 :: (label: *u8, v: *[3] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat3@ImGui@@YA_NPEBDQEAM0H@Z";
}

#if OS != .WINDOWS {
    InputFloat4 :: (label: *u8, v: *[4] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui11InputFloat4EPKcPfS1_i";
}

#if OS == .WINDOWS {
    InputFloat4 :: (label: *u8, v: *[4] float, format: *u8 = "%.3f", flags: InputTextFlags = .None) -> bool #foreign imgui "?InputFloat4@ImGui@@YA_NPEBDQEAM0H@Z";
}

#if OS != .WINDOWS {
    InputInt :: (label: *u8, v: *s32, step: s32 = 1, step_fast: s32 = 100, flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui8InputIntEPKcPiiii";
}

#if OS == .WINDOWS {
    InputInt :: (label: *u8, v: *s32, step: s32 = 1, step_fast: s32 = 100, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt@ImGui@@YA_NPEBDPEAHHHH@Z";
}

#if OS != .WINDOWS {
    InputInt2 :: (label: *u8, v: *[2] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui9InputInt2EPKcPii";
}

#if OS == .WINDOWS {
    InputInt2 :: (label: *u8, v: *[2] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt2@ImGui@@YA_NPEBDQEAHH@Z";
}

#if OS != .WINDOWS {
    InputInt3 :: (label: *u8, v: *[3] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui9InputInt3EPKcPii";
}

#if OS == .WINDOWS {
    InputInt3 :: (label: *u8, v: *[3] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt3@ImGui@@YA_NPEBDQEAHH@Z";
}

#if OS != .WINDOWS {
    InputInt4 :: (label: *u8, v: *[4] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui9InputInt4EPKcPii";
}

#if OS == .WINDOWS {
    InputInt4 :: (label: *u8, v: *[4] s32, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputInt4@ImGui@@YA_NPEBDQEAHH@Z";
}

#if OS != .WINDOWS {
    InputScalar :: (label: *u8, data_type: DataType, p_data: *void, p_step: *void = null, p_step_fast: *void = null, format: *u8 = null, flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui11InputScalarEPKciPvPKvS4_S1_i";
}

#if OS == .WINDOWS {
    InputScalar :: (label: *u8, data_type: DataType, p_data: *void, p_step: *void = null, p_step_fast: *void = null, format: *u8 = null, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputScalar@ImGui@@YA_NPEBDHPEAXPEBX20H@Z";
}

#if OS != .WINDOWS {
    InputScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, p_step: *void = null, p_step_fast: *void = null, format: *u8 = null, flags: InputTextFlags = .None) -> bool #foreign imgui "_ZN5ImGui12InputScalarNEPKciPviPKvS4_S1_i";
}

#if OS == .WINDOWS {
    InputScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, p_step: *void = null, p_step_fast: *void = null, format: *u8 = null, flags: InputTextFlags = .None) -> bool #foreign imgui "?InputScalarN@ImGui@@YA_NPEBDHPEAXHPEBX20H@Z";
}

#if OS != .WINDOWS {
    InputText :: (label: *u8, buf: *u8, buf_size: u64, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "_ZN5ImGui9InputTextEPKcPcmiPFiP26ImGuiInputTextCallbackDataEPv";
}

#if OS == .WINDOWS {
    InputText :: (label: *u8, buf: *u8, buf_size: u64, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "?InputText@ImGui@@YA_NPEBDPEAD_KHP6AHPEAUImGuiInputTextCallbackData@@@ZPEAX@Z";
}

#if OS != .WINDOWS {
    InputTextMultiline :: (label: *u8, buf: *u8, buf_size: u64, size: *ImVec2, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "_ZN5ImGui18InputTextMultilineEPKcPcmRK6ImVec2iPFiP26ImGuiInputTextCallbackDataEPv";
}

#if OS == .WINDOWS {
    InputTextMultiline :: (label: *u8, buf: *u8, buf_size: u64, size: *ImVec2, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "?InputTextMultiline@ImGui@@YA_NPEBDPEAD_KAEBUImVec2@@HP6AHPEAUImGuiInputTextCallbackData@@@ZPEAX@Z";
}

#if OS != .WINDOWS {
    InputTextWithHint :: (label: *u8, hint: *u8, buf: *u8, buf_size: u64, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "_ZN5ImGui17InputTextWithHintEPKcS1_PcmiPFiP26ImGuiInputTextCallbackDataEPv";
}

#if OS == .WINDOWS {
    InputTextWithHint :: (label: *u8, hint: *u8, buf: *u8, buf_size: u64, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #foreign imgui "?InputTextWithHint@ImGui@@YA_NPEBD0PEAD_KHP6AHPEAUImGuiInputTextCallbackData@@@ZPEAX@Z";
}

#if OS != .WINDOWS {
    InvisibleButton :: (str_id: *u8, size: *ImVec2, flags: ButtonFlags = .None) -> bool #foreign imgui "_ZN5ImGui15InvisibleButtonEPKcRK6ImVec2i";
}

#if OS == .WINDOWS {
    InvisibleButton :: (str_id: *u8, size: *ImVec2, flags: ButtonFlags = .None) -> bool #foreign imgui "?InvisibleButton@ImGui@@YA_NPEBDAEBUImVec2@@H@Z";
}

#if OS != .WINDOWS {
    IsAnyItemActive :: () -> bool #foreign imgui "_ZN5ImGui15IsAnyItemActiveEv";
}

#if OS == .WINDOWS {
    IsAnyItemActive :: () -> bool #foreign imgui "?IsAnyItemActive@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsAnyItemFocused :: () -> bool #foreign imgui "_ZN5ImGui16IsAnyItemFocusedEv";
}

#if OS == .WINDOWS {
    IsAnyItemFocused :: () -> bool #foreign imgui "?IsAnyItemFocused@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsAnyItemHovered :: () -> bool #foreign imgui "_ZN5ImGui16IsAnyItemHoveredEv";
}

#if OS == .WINDOWS {
    IsAnyItemHovered :: () -> bool #foreign imgui "?IsAnyItemHovered@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsAnyMouseDown :: () -> bool #foreign imgui "_ZN5ImGui14IsAnyMouseDownEv";
}

#if OS == .WINDOWS {
    IsAnyMouseDown :: () -> bool #foreign imgui "?IsAnyMouseDown@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemActivated :: () -> bool #foreign imgui "_ZN5ImGui15IsItemActivatedEv";
}

#if OS == .WINDOWS {
    IsItemActivated :: () -> bool #foreign imgui "?IsItemActivated@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemActive :: () -> bool #foreign imgui "_ZN5ImGui12IsItemActiveEv";
}

#if OS == .WINDOWS {
    IsItemActive :: () -> bool #foreign imgui "?IsItemActive@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemClicked :: (mouse_button: MouseButton = .Left) -> bool #foreign imgui "_ZN5ImGui13IsItemClickedEi";
}

#if OS == .WINDOWS {
    IsItemClicked :: (mouse_button: MouseButton = .Left) -> bool #foreign imgui "?IsItemClicked@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    IsItemDeactivated :: () -> bool #foreign imgui "_ZN5ImGui17IsItemDeactivatedEv";
}

#if OS == .WINDOWS {
    IsItemDeactivated :: () -> bool #foreign imgui "?IsItemDeactivated@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemDeactivatedAfterEdit :: () -> bool #foreign imgui "_ZN5ImGui26IsItemDeactivatedAfterEditEv";
}

#if OS == .WINDOWS {
    IsItemDeactivatedAfterEdit :: () -> bool #foreign imgui "?IsItemDeactivatedAfterEdit@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemEdited :: () -> bool #foreign imgui "_ZN5ImGui12IsItemEditedEv";
}

#if OS == .WINDOWS {
    IsItemEdited :: () -> bool #foreign imgui "?IsItemEdited@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemFocused :: () -> bool #foreign imgui "_ZN5ImGui13IsItemFocusedEv";
}

#if OS == .WINDOWS {
    IsItemFocused :: () -> bool #foreign imgui "?IsItemFocused@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemHovered :: (flags: HoveredFlags = .None) -> bool #foreign imgui "_ZN5ImGui13IsItemHoveredEi";
}

#if OS == .WINDOWS {
    IsItemHovered :: (flags: HoveredFlags = .None) -> bool #foreign imgui "?IsItemHovered@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    IsItemToggledOpen :: () -> bool #foreign imgui "_ZN5ImGui17IsItemToggledOpenEv";
}

#if OS == .WINDOWS {
    IsItemToggledOpen :: () -> bool #foreign imgui "?IsItemToggledOpen@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsItemVisible :: () -> bool #foreign imgui "_ZN5ImGui13IsItemVisibleEv";
}

#if OS == .WINDOWS {
    IsItemVisible :: () -> bool #foreign imgui "?IsItemVisible@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsKeyDown :: (key: Key) -> bool #foreign imgui "_ZN5ImGui9IsKeyDownE8ImGuiKey";
}

#if OS == .WINDOWS {
    IsKeyDown :: (key: Key) -> bool #foreign imgui "?IsKeyDown@ImGui@@YA_NW4ImGuiKey@@@Z";
}

#if OS != .WINDOWS {
    IsKeyPressed :: (key: Key, repeat := true) -> bool #foreign imgui "_ZN5ImGui12IsKeyPressedE8ImGuiKeyb";
}

#if OS == .WINDOWS {
    IsKeyPressed :: (key: Key, repeat := true) -> bool #foreign imgui "?IsKeyPressed@ImGui@@YA_NW4ImGuiKey@@_N@Z";
}

#if OS != .WINDOWS {
    IsKeyReleased :: (key: Key) -> bool #foreign imgui "_ZN5ImGui13IsKeyReleasedE8ImGuiKey";
}

#if OS == .WINDOWS {
    IsKeyReleased :: (key: Key) -> bool #foreign imgui "?IsKeyReleased@ImGui@@YA_NW4ImGuiKey@@@Z";
}

#if OS != .WINDOWS {
    IsMouseClicked :: (button: MouseButton, repeat := false) -> bool #foreign imgui "_ZN5ImGui14IsMouseClickedEib";
}

#if OS == .WINDOWS {
    IsMouseClicked :: (button: MouseButton, repeat := false) -> bool #foreign imgui "?IsMouseClicked@ImGui@@YA_NH_N@Z";
}

#if OS != .WINDOWS {
    IsMouseDoubleClicked :: (button: MouseButton) -> bool #foreign imgui "_ZN5ImGui20IsMouseDoubleClickedEi";
}

#if OS == .WINDOWS {
    IsMouseDoubleClicked :: (button: MouseButton) -> bool #foreign imgui "?IsMouseDoubleClicked@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    IsMouseDown :: (button: MouseButton) -> bool #foreign imgui "_ZN5ImGui11IsMouseDownEi";
}

#if OS == .WINDOWS {
    IsMouseDown :: (button: MouseButton) -> bool #foreign imgui "?IsMouseDown@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    IsMouseDragging :: (button: MouseButton, lock_threshold: float = -1.0) -> bool #foreign imgui "_ZN5ImGui15IsMouseDraggingEif";
}

#if OS == .WINDOWS {
    IsMouseDragging :: (button: MouseButton, lock_threshold: float = -1.0) -> bool #foreign imgui "?IsMouseDragging@ImGui@@YA_NHM@Z";
}

#if OS != .WINDOWS {
    IsMouseHoveringRect :: (r_min: *ImVec2, r_max: *ImVec2, clip := true) -> bool #foreign imgui "_ZN5ImGui19IsMouseHoveringRectERK6ImVec2S2_b";
}

#if OS == .WINDOWS {
    IsMouseHoveringRect :: (r_min: *ImVec2, r_max: *ImVec2, clip := true) -> bool #foreign imgui "?IsMouseHoveringRect@ImGui@@YA_NAEBUImVec2@@0_N@Z";
}

#if OS != .WINDOWS {
    IsMousePosValid :: (mouse_pos: *ImVec2 = null) -> bool #foreign imgui "_ZN5ImGui15IsMousePosValidEPK6ImVec2";
}

#if OS == .WINDOWS {
    IsMousePosValid :: (mouse_pos: *ImVec2 = null) -> bool #foreign imgui "?IsMousePosValid@ImGui@@YA_NPEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    IsMouseReleased :: (button: MouseButton) -> bool #foreign imgui "_ZN5ImGui15IsMouseReleasedEi";
}

#if OS == .WINDOWS {
    IsMouseReleased :: (button: MouseButton) -> bool #foreign imgui "?IsMouseReleased@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    IsPopupOpen :: (str_id: *u8, flags: PopupFlags = .None) -> bool #foreign imgui "_ZN5ImGui11IsPopupOpenEPKci";
}

#if OS == .WINDOWS {
    IsPopupOpen :: (str_id: *u8, flags: PopupFlags = .None) -> bool #foreign imgui "?IsPopupOpen@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    IsRectVisible :: (size: *ImVec2) -> bool #foreign imgui "_ZN5ImGui13IsRectVisibleERK6ImVec2";
}

#if OS != .WINDOWS {
    IsRectVisible :: (rect_min: *ImVec2, rect_max: *ImVec2) -> bool #foreign imgui "_ZN5ImGui13IsRectVisibleERK6ImVec2S2_";
}

#if OS == .WINDOWS {
    IsRectVisible :: (size: *ImVec2) -> bool #foreign imgui "?IsRectVisible@ImGui@@YA_NAEBUImVec2@@@Z";
}

#if OS == .WINDOWS {
    IsRectVisible :: (rect_min: *ImVec2, rect_max: *ImVec2) -> bool #foreign imgui "?IsRectVisible@ImGui@@YA_NAEBUImVec2@@0@Z";
}

#if OS != .WINDOWS {
    IsWindowAppearing :: () -> bool #foreign imgui "_ZN5ImGui17IsWindowAppearingEv";
}

#if OS == .WINDOWS {
    IsWindowAppearing :: () -> bool #foreign imgui "?IsWindowAppearing@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsWindowCollapsed :: () -> bool #foreign imgui "_ZN5ImGui17IsWindowCollapsedEv";
}

#if OS == .WINDOWS {
    IsWindowCollapsed :: () -> bool #foreign imgui "?IsWindowCollapsed@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    IsWindowFocused :: (flags: FocusedFlags = .None) -> bool #foreign imgui "_ZN5ImGui15IsWindowFocusedEi";
}

#if OS == .WINDOWS {
    IsWindowFocused :: (flags: FocusedFlags = .None) -> bool #foreign imgui "?IsWindowFocused@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    IsWindowHovered :: (flags: HoveredFlags = .None) -> bool #foreign imgui "_ZN5ImGui15IsWindowHoveredEi";
}

#if OS == .WINDOWS {
    IsWindowHovered :: (flags: HoveredFlags = .None) -> bool #foreign imgui "?IsWindowHovered@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    LabelText_CFormat :: (label: *u8, fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui9LabelTextEPKcS1_z";
}

#if OS == .WINDOWS {
    LabelText_CFormat :: (label: *u8, fmt: *u8, __args: ..Any) -> void #foreign imgui "?LabelText@ImGui@@YAXPEBD0ZZ";
}

#if OS != .WINDOWS {
    ListBox :: (label: *u8, current_item: *s32, items: **u8, items_count: s32, height_in_items: s32 = -1) -> bool #foreign imgui "_ZN5ImGui7ListBoxEPKcPiPKS1_ii";
}

#if OS != .WINDOWS {
    ListBox :: (label: *u8, current_item: *s32, items_getter: #type (data: *void, idx: s32, out_text: **u8) -> bool #c_call, data: *void, items_count: s32, height_in_items: s32 = -1) -> bool #foreign imgui "_ZN5ImGui7ListBoxEPKcPiPFbPviPS1_ES3_ii";
}

#if OS == .WINDOWS {
    ListBox :: (label: *u8, current_item: *s32, items: **u8, items_count: s32, height_in_items: s32 = -1) -> bool #foreign imgui "?ListBox@ImGui@@YA_NPEBDPEAHQEBQEBDHH@Z";
}

#if OS == .WINDOWS {
    ListBox :: (label: *u8, current_item: *s32, items_getter: #type (data: *void, idx: s32, out_text: **u8) -> bool #c_call, data: *void, items_count: s32, height_in_items: s32 = -1) -> bool #foreign imgui "?ListBox@ImGui@@YA_NPEBDPEAHP6A_NPEAXHPEAPEBD@Z2HH@Z";
}

#if OS != .WINDOWS {
    LoadIniSettingsFromDisk :: (ini_filename: *u8) -> void #foreign imgui "_ZN5ImGui23LoadIniSettingsFromDiskEPKc";
}

#if OS == .WINDOWS {
    LoadIniSettingsFromDisk :: (ini_filename: *u8) -> void #foreign imgui "?LoadIniSettingsFromDisk@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    LoadIniSettingsFromMemory :: (ini_data: *u8, ini_size: u64 = 0) -> void #foreign imgui "_ZN5ImGui25LoadIniSettingsFromMemoryEPKcm";
}

#if OS == .WINDOWS {
    LoadIniSettingsFromMemory :: (ini_data: *u8, ini_size: u64 = 0) -> void #foreign imgui "?LoadIniSettingsFromMemory@ImGui@@YAXPEBD_K@Z";
}

#if OS != .WINDOWS {
    LogButtons :: () -> void #foreign imgui "_ZN5ImGui10LogButtonsEv";
}

#if OS == .WINDOWS {
    LogButtons :: () -> void #foreign imgui "?LogButtons@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    LogFinish :: () -> void #foreign imgui "_ZN5ImGui9LogFinishEv";
}

#if OS == .WINDOWS {
    LogFinish :: () -> void #foreign imgui "?LogFinish@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    LogText_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui7LogTextEPKcz";
}

#if OS == .WINDOWS {
    LogText_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?LogText@ImGui@@YAXPEBDZZ";
}

#if OS != .WINDOWS {
    LogToClipboard :: (auto_open_depth: s32 = -1) -> void #foreign imgui "_ZN5ImGui14LogToClipboardEi";
}

#if OS == .WINDOWS {
    LogToClipboard :: (auto_open_depth: s32 = -1) -> void #foreign imgui "?LogToClipboard@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    LogToFile :: (auto_open_depth: s32 = -1, filename: *u8 = null) -> void #foreign imgui "_ZN5ImGui9LogToFileEiPKc";
}

#if OS == .WINDOWS {
    LogToFile :: (auto_open_depth: s32 = -1, filename: *u8 = null) -> void #foreign imgui "?LogToFile@ImGui@@YAXHPEBD@Z";
}

#if OS != .WINDOWS {
    LogToTTY :: (auto_open_depth: s32 = -1) -> void #foreign imgui "_ZN5ImGui8LogToTTYEi";
}

#if OS == .WINDOWS {
    LogToTTY :: (auto_open_depth: s32 = -1) -> void #foreign imgui "?LogToTTY@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    MemAlloc :: (size: u64) -> *void #foreign imgui "_ZN5ImGui8MemAllocEm";
}

#if OS == .WINDOWS {
    MemAlloc :: (size: u64) -> *void #foreign imgui "?MemAlloc@ImGui@@YAPEAX_K@Z";
}

#if OS != .WINDOWS {
    MemFree :: (ptr: *void) -> void #foreign imgui "_ZN5ImGui7MemFreeEPv";
}

#if OS == .WINDOWS {
    MemFree :: (ptr: *void) -> void #foreign imgui "?MemFree@ImGui@@YAXPEAX@Z";
}

#if OS != .WINDOWS {
    MenuItem :: (label: *u8, shortcut: *u8 = null, selected := false, enabled := true) -> bool #foreign imgui "_ZN5ImGui8MenuItemEPKcS1_bb";
}

#if OS != .WINDOWS {
    MenuItem :: (label: *u8, shortcut: *u8, p_selected: *bool, enabled := true) -> bool #foreign imgui "_ZN5ImGui8MenuItemEPKcS1_Pbb";
}

#if OS == .WINDOWS {
    MenuItem :: (label: *u8, shortcut: *u8 = null, selected := false, enabled := true) -> bool #foreign imgui "?MenuItem@ImGui@@YA_NPEBD0_N1@Z";
}

#if OS == .WINDOWS {
    MenuItem :: (label: *u8, shortcut: *u8, p_selected: *bool, enabled := true) -> bool #foreign imgui "?MenuItem@ImGui@@YA_NPEBD0PEA_N_N@Z";
}

#if OS != .WINDOWS {
    NewFrame :: () -> void #foreign imgui "_ZN5ImGui8NewFrameEv";
}

#if OS == .WINDOWS {
    NewFrame :: () -> void #foreign imgui "?NewFrame@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    NewLine :: () -> void #foreign imgui "_ZN5ImGui7NewLineEv";
}

#if OS == .WINDOWS {
    NewLine :: () -> void #foreign imgui "?NewLine@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    NextColumn :: () -> void #foreign imgui "_ZN5ImGui10NextColumnEv";
}

#if OS == .WINDOWS {
    NextColumn :: () -> void #foreign imgui "?NextColumn@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    OpenPopup :: (str_id: *u8, popup_flags: PopupFlags = .None) -> void #foreign imgui "_ZN5ImGui9OpenPopupEPKci";
}

#if OS != .WINDOWS {
    OpenPopup :: (id: ID, popup_flags: PopupFlags = .None) -> void #foreign imgui "_ZN5ImGui9OpenPopupEji";
}

#if OS == .WINDOWS {
    OpenPopup :: (str_id: *u8, popup_flags: PopupFlags = .None) -> void #foreign imgui "?OpenPopup@ImGui@@YAXPEBDH@Z";
}

#if OS == .WINDOWS {
    OpenPopup :: (id: ID, popup_flags: PopupFlags = .None) -> void #foreign imgui "?OpenPopup@ImGui@@YAXIH@Z";
}

#if OS != .WINDOWS {
    OpenPopupOnItemClick :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> void #foreign imgui "_ZN5ImGui20OpenPopupOnItemClickEPKci";
}

#if OS == .WINDOWS {
    OpenPopupOnItemClick :: (str_id: *u8 = null, popup_flags: PopupFlags = .MouseButtonRight) -> void #foreign imgui "?OpenPopupOnItemClick@ImGui@@YAXPEBDH@Z";
}

#if OS != .WINDOWS {
    PlotHistogram :: (label: *u8, values: *float, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }, stride: s32 = 4) -> void #foreign imgui "_ZN5ImGui13PlotHistogramEPKcPKfiiS1_ff6ImVec2i";
}

#if OS != .WINDOWS {
    PlotHistogram :: (label: *u8, values_getter: #type (data: *void, idx: s32) -> float #c_call, data: *void, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }) -> void #foreign imgui "_ZN5ImGui13PlotHistogramEPKcPFfPviES2_iiS1_ff6ImVec2";
}

#if OS == .WINDOWS {
    PlotHistogram :: (label: *u8, values: *float, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }, stride: s32 = 4) -> void #foreign imgui "?PlotHistogram@ImGui@@YAXPEBDPEBMHH0MMUImVec2@@H@Z";
}

#if OS == .WINDOWS {
    PlotHistogram :: (label: *u8, values_getter: #type (data: *void, idx: s32) -> float #c_call, data: *void, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }) -> void #foreign imgui "?PlotHistogram@ImGui@@YAXPEBDP6AMPEAXH@Z1HH0MMUImVec2@@@Z";
}

#if OS != .WINDOWS {
    PlotLines :: (label: *u8, values: *float, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }, stride: s32 = 4) -> void #foreign imgui "_ZN5ImGui9PlotLinesEPKcPKfiiS1_ff6ImVec2i";
}

#if OS != .WINDOWS {
    PlotLines :: (label: *u8, values_getter: #type (data: *void, idx: s32) -> float #c_call, data: *void, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }) -> void #foreign imgui "_ZN5ImGui9PlotLinesEPKcPFfPviES2_iiS1_ff6ImVec2";
}

#if OS == .WINDOWS {
    PlotLines :: (label: *u8, values: *float, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }, stride: s32 = 4) -> void #foreign imgui "?PlotLines@ImGui@@YAXPEBDPEBMHH0MMUImVec2@@H@Z";
}

#if OS == .WINDOWS {
    PlotLines :: (label: *u8, values_getter: #type (data: *void, idx: s32) -> float #c_call, data: *void, values_count: s32, values_offset: s32 = 0, overlay_text: *u8 = null, scale_min: float = FLOAT32_MAX, scale_max: float = FLOAT32_MAX, graph_size: ImVec2 = .{ 0, 0 }) -> void #foreign imgui "?PlotLines@ImGui@@YAXPEBDP6AMPEAXH@Z1HH0MMUImVec2@@@Z";
}

#if OS != .WINDOWS {
    PopButtonRepeat :: () -> void #foreign imgui "_ZN5ImGui15PopButtonRepeatEv";
}

#if OS == .WINDOWS {
    PopButtonRepeat :: () -> void #foreign imgui "?PopButtonRepeat@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    PopClipRect :: () -> void #foreign imgui "_ZN5ImGui11PopClipRectEv";
}

#if OS == .WINDOWS {
    PopClipRect :: () -> void #foreign imgui "?PopClipRect@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    PopFont :: () -> void #foreign imgui "_ZN5ImGui7PopFontEv";
}

#if OS == .WINDOWS {
    PopFont :: () -> void #foreign imgui "?PopFont@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    PopID :: () -> void #foreign imgui "_ZN5ImGui5PopIDEv";
}

#if OS == .WINDOWS {
    PopID :: () -> void #foreign imgui "?PopID@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    PopItemWidth :: () -> void #foreign imgui "_ZN5ImGui12PopItemWidthEv";
}

#if OS == .WINDOWS {
    PopItemWidth :: () -> void #foreign imgui "?PopItemWidth@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    PopStyleColor :: (count: s32 = 1) -> void #foreign imgui "_ZN5ImGui13PopStyleColorEi";
}

#if OS == .WINDOWS {
    PopStyleColor :: (count: s32 = 1) -> void #foreign imgui "?PopStyleColor@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    PopStyleVar :: (count: s32 = 1) -> void #foreign imgui "_ZN5ImGui11PopStyleVarEi";
}

#if OS == .WINDOWS {
    PopStyleVar :: (count: s32 = 1) -> void #foreign imgui "?PopStyleVar@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    PopTabStop :: () -> void #foreign imgui "_ZN5ImGui10PopTabStopEv";
}

#if OS == .WINDOWS {
    PopTabStop :: () -> void #foreign imgui "?PopTabStop@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    PopTextWrapPos :: () -> void #foreign imgui "_ZN5ImGui14PopTextWrapPosEv";
}

#if OS == .WINDOWS {
    PopTextWrapPos :: () -> void #foreign imgui "?PopTextWrapPos@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    ProgressBar :: (fraction: float, size_arg: *ImVec2, overlay: *u8 = null) -> void #foreign imgui "_ZN5ImGui11ProgressBarEfRK6ImVec2PKc";
}

#if OS == .WINDOWS {
    ProgressBar :: (fraction: float, size_arg: *ImVec2, overlay: *u8 = null) -> void #foreign imgui "?ProgressBar@ImGui@@YAXMAEBUImVec2@@PEBD@Z";
}

#if OS != .WINDOWS {
    PushButtonRepeat :: (repeat: bool) -> void #foreign imgui "_ZN5ImGui16PushButtonRepeatEb";
}

#if OS == .WINDOWS {
    PushButtonRepeat :: (repeat: bool) -> void #foreign imgui "?PushButtonRepeat@ImGui@@YAX_N@Z";
}

#if OS != .WINDOWS {
    PushClipRect :: (clip_rect_min: *ImVec2, clip_rect_max: *ImVec2, intersect_with_current_clip_rect: bool) -> void #foreign imgui "_ZN5ImGui12PushClipRectERK6ImVec2S2_b";
}

#if OS == .WINDOWS {
    PushClipRect :: (clip_rect_min: *ImVec2, clip_rect_max: *ImVec2, intersect_with_current_clip_rect: bool) -> void #foreign imgui "?PushClipRect@ImGui@@YAXAEBUImVec2@@0_N@Z";
}

#if OS != .WINDOWS {
    PushFont :: (font: *ImFont) -> void #foreign imgui "_ZN5ImGui8PushFontEP6ImFont";
}

#if OS == .WINDOWS {
    PushFont :: (font: *ImFont) -> void #foreign imgui "?PushFont@ImGui@@YAXPEAUImFont@@@Z";
}

#if OS != .WINDOWS {
    PushID :: (str_id: *u8) -> void #foreign imgui "_ZN5ImGui6PushIDEPKc";
}

#if OS != .WINDOWS {
    PushID :: (str_id_begin: *u8, str_id_end: *u8) -> void #foreign imgui "_ZN5ImGui6PushIDEPKcS1_";
}

#if OS != .WINDOWS {
    PushID :: (ptr_id: *void) -> void #foreign imgui "_ZN5ImGui6PushIDEPKv";
}

#if OS != .WINDOWS {
    PushID :: (int_id: s32) -> void #foreign imgui "_ZN5ImGui6PushIDEi";
}

#if OS == .WINDOWS {
    PushID :: (str_id: *u8) -> void #foreign imgui "?PushID@ImGui@@YAXPEBD@Z";
}

#if OS == .WINDOWS {
    PushID :: (str_id_begin: *u8, str_id_end: *u8) -> void #foreign imgui "?PushID@ImGui@@YAXPEBD0@Z";
}

#if OS == .WINDOWS {
    PushID :: (ptr_id: *void) -> void #foreign imgui "?PushID@ImGui@@YAXPEBX@Z";
}

#if OS == .WINDOWS {
    PushID :: (int_id: s32) -> void #foreign imgui "?PushID@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    PushItemWidth :: (item_width: float) -> void #foreign imgui "_ZN5ImGui13PushItemWidthEf";
}

#if OS == .WINDOWS {
    PushItemWidth :: (item_width: float) -> void #foreign imgui "?PushItemWidth@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    PushStyleColor :: (idx: Col, col: ImU32) -> void #foreign imgui "_ZN5ImGui14PushStyleColorEij";
}

#if OS != .WINDOWS {
    PushStyleColor :: (idx: Col, col: *ImVec4) -> void #foreign imgui "_ZN5ImGui14PushStyleColorEiRK6ImVec4";
}

#if OS == .WINDOWS {
    PushStyleColor :: (idx: Col, col: ImU32) -> void #foreign imgui "?PushStyleColor@ImGui@@YAXHI@Z";
}

#if OS == .WINDOWS {
    PushStyleColor :: (idx: Col, col: *ImVec4) -> void #foreign imgui "?PushStyleColor@ImGui@@YAXHAEBUImVec4@@@Z";
}

#if OS != .WINDOWS {
    PushStyleVar :: (idx: StyleVar, val: float) -> void #foreign imgui "_ZN5ImGui12PushStyleVarEif";
}

#if OS != .WINDOWS {
    PushStyleVar :: (idx: StyleVar, val: *ImVec2) -> void #foreign imgui "_ZN5ImGui12PushStyleVarEiRK6ImVec2";
}

#if OS == .WINDOWS {
    PushStyleVar :: (idx: StyleVar, val: float) -> void #foreign imgui "?PushStyleVar@ImGui@@YAXHM@Z";
}

#if OS == .WINDOWS {
    PushStyleVar :: (idx: StyleVar, val: *ImVec2) -> void #foreign imgui "?PushStyleVar@ImGui@@YAXHAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    PushTabStop :: (tab_stop: bool) -> void #foreign imgui "_ZN5ImGui11PushTabStopEb";
}

#if OS == .WINDOWS {
    PushTabStop :: (tab_stop: bool) -> void #foreign imgui "?PushTabStop@ImGui@@YAX_N@Z";
}

#if OS != .WINDOWS {
    PushTextWrapPos :: (wrap_local_pos_x: float = 0.0) -> void #foreign imgui "_ZN5ImGui15PushTextWrapPosEf";
}

#if OS == .WINDOWS {
    PushTextWrapPos :: (wrap_local_pos_x: float = 0.0) -> void #foreign imgui "?PushTextWrapPos@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    RadioButton :: (label: *u8, active: bool) -> bool #foreign imgui "_ZN5ImGui11RadioButtonEPKcb";
}

#if OS != .WINDOWS {
    RadioButton :: (label: *u8, v: *s32, v_button: s32) -> bool #foreign imgui "_ZN5ImGui11RadioButtonEPKcPii";
}

#if OS == .WINDOWS {
    RadioButton :: (label: *u8, active: bool) -> bool #foreign imgui "?RadioButton@ImGui@@YA_NPEBD_N@Z";
}

#if OS == .WINDOWS {
    RadioButton :: (label: *u8, v: *s32, v_button: s32) -> bool #foreign imgui "?RadioButton@ImGui@@YA_NPEBDPEAHH@Z";
}

#if OS != .WINDOWS {
    Render :: () -> void #foreign imgui "_ZN5ImGui6RenderEv";
}

#if OS == .WINDOWS {
    Render :: () -> void #foreign imgui "?Render@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    ResetMouseDragDelta :: (button: MouseButton = .Left) -> void #foreign imgui "_ZN5ImGui19ResetMouseDragDeltaEi";
}

#if OS == .WINDOWS {
    ResetMouseDragDelta :: (button: MouseButton = .Left) -> void #foreign imgui "?ResetMouseDragDelta@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    SameLine :: (offset_from_start_x: float = 0.0, spacing: float = -1.0) -> void #foreign imgui "_ZN5ImGui8SameLineEff";
}

#if OS == .WINDOWS {
    SameLine :: (offset_from_start_x: float = 0.0, spacing: float = -1.0) -> void #foreign imgui "?SameLine@ImGui@@YAXMM@Z";
}

#if OS != .WINDOWS {
    SaveIniSettingsToDisk :: (ini_filename: *u8) -> void #foreign imgui "_ZN5ImGui21SaveIniSettingsToDiskEPKc";
}

#if OS == .WINDOWS {
    SaveIniSettingsToDisk :: (ini_filename: *u8) -> void #foreign imgui "?SaveIniSettingsToDisk@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    SaveIniSettingsToMemory :: (out_ini_size: *u64 = null) -> *u8 #foreign imgui "_ZN5ImGui23SaveIniSettingsToMemoryEPm";
}

#if OS == .WINDOWS {
    SaveIniSettingsToMemory :: (out_ini_size: *u64 = null) -> *u8 #foreign imgui "?SaveIniSettingsToMemory@ImGui@@YAPEBDPEA_K@Z";
}

#if OS != .WINDOWS {
    Selectable :: (label: *u8, selected := false, flags: SelectableFlags = .None, size: *ImVec2) -> bool #foreign imgui "_ZN5ImGui10SelectableEPKcbiRK6ImVec2";
}

#if OS != .WINDOWS {
    Selectable :: (label: *u8, p_selected: *bool, flags: SelectableFlags = .None, size: *ImVec2) -> bool #foreign imgui "_ZN5ImGui10SelectableEPKcPbiRK6ImVec2";
}

#if OS == .WINDOWS {
    Selectable :: (label: *u8, selected := false, flags: SelectableFlags = .None, size: *ImVec2) -> bool #foreign imgui "?Selectable@ImGui@@YA_NPEBD_NHAEBUImVec2@@@Z";
}

#if OS == .WINDOWS {
    Selectable :: (label: *u8, p_selected: *bool, flags: SelectableFlags = .None, size: *ImVec2) -> bool #foreign imgui "?Selectable@ImGui@@YA_NPEBDPEA_NHAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    Separator :: () -> void #foreign imgui "_ZN5ImGui9SeparatorEv";
}

#if OS == .WINDOWS {
    Separator :: () -> void #foreign imgui "?Separator@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    SeparatorText :: (label: *u8) -> void #foreign imgui "_ZN5ImGui13SeparatorTextEPKc";
}

#if OS == .WINDOWS {
    SeparatorText :: (label: *u8) -> void #foreign imgui "?SeparatorText@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    SetAllocatorFunctions :: (alloc_func: MemAllocFunc, free_func: MemFreeFunc, user_data: *void = null) -> void #foreign imgui "_ZN5ImGui21SetAllocatorFunctionsEPFPvmS0_EPFvS0_S0_ES0_";
}

#if OS == .WINDOWS {
    SetAllocatorFunctions :: (alloc_func: MemAllocFunc, free_func: MemFreeFunc, user_data: *void = null) -> void #foreign imgui "?SetAllocatorFunctions@ImGui@@YAXP6APEAX_KPEAX@ZP6AX11@Z1@Z";
}

#if OS != .WINDOWS {
    SetClipboardText :: (text: *u8) -> void #foreign imgui "_ZN5ImGui16SetClipboardTextEPKc";
}

#if OS == .WINDOWS {
    SetClipboardText :: (text: *u8) -> void #foreign imgui "?SetClipboardText@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    SetColorEditOptions :: (flags: ColorEditFlags) -> void #foreign imgui "_ZN5ImGui19SetColorEditOptionsEi";
}

#if OS == .WINDOWS {
    SetColorEditOptions :: (flags: ColorEditFlags) -> void #foreign imgui "?SetColorEditOptions@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    SetColumnOffset :: (column_index: s32, offset_x: float) -> void #foreign imgui "_ZN5ImGui15SetColumnOffsetEif";
}

#if OS == .WINDOWS {
    SetColumnOffset :: (column_index: s32, offset_x: float) -> void #foreign imgui "?SetColumnOffset@ImGui@@YAXHM@Z";
}

#if OS != .WINDOWS {
    SetColumnWidth :: (column_index: s32, width: float) -> void #foreign imgui "_ZN5ImGui14SetColumnWidthEif";
}

#if OS == .WINDOWS {
    SetColumnWidth :: (column_index: s32, width: float) -> void #foreign imgui "?SetColumnWidth@ImGui@@YAXHM@Z";
}

#if OS != .WINDOWS {
    SetCurrentContext :: (ctx: *ImGuiContext) -> void #foreign imgui "_ZN5ImGui17SetCurrentContextEP12ImGuiContext";
}

#if OS == .WINDOWS {
    SetCurrentContext :: (ctx: *ImGuiContext) -> void #foreign imgui "?SetCurrentContext@ImGui@@YAXPEAUImGuiContext@@@Z";
}

#if OS != .WINDOWS {
    SetCursorPos :: (local_pos: *ImVec2) -> void #foreign imgui "_ZN5ImGui12SetCursorPosERK6ImVec2";
}

#if OS == .WINDOWS {
    SetCursorPos :: (local_pos: *ImVec2) -> void #foreign imgui "?SetCursorPos@ImGui@@YAXAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    SetCursorPosX :: (local_x: float) -> void #foreign imgui "_ZN5ImGui13SetCursorPosXEf";
}

#if OS == .WINDOWS {
    SetCursorPosX :: (local_x: float) -> void #foreign imgui "?SetCursorPosX@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetCursorPosY :: (local_y: float) -> void #foreign imgui "_ZN5ImGui13SetCursorPosYEf";
}

#if OS == .WINDOWS {
    SetCursorPosY :: (local_y: float) -> void #foreign imgui "?SetCursorPosY@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetCursorScreenPos :: (pos: *ImVec2) -> void #foreign imgui "_ZN5ImGui18SetCursorScreenPosERK6ImVec2";
}

#if OS == .WINDOWS {
    SetCursorScreenPos :: (pos: *ImVec2) -> void #foreign imgui "?SetCursorScreenPos@ImGui@@YAXAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    SetDragDropPayload :: (type: *u8, data: *void, sz: u64, cond: Cond = .None) -> bool #foreign imgui "_ZN5ImGui18SetDragDropPayloadEPKcPKvmi";
}

#if OS == .WINDOWS {
    SetDragDropPayload :: (type: *u8, data: *void, sz: u64, cond: Cond = .None) -> bool #foreign imgui "?SetDragDropPayload@ImGui@@YA_NPEBDPEBX_KH@Z";
}

#if OS != .WINDOWS {
    SetItemAllowOverlap :: () -> void #foreign imgui "_ZN5ImGui19SetItemAllowOverlapEv";
}

#if OS == .WINDOWS {
    SetItemAllowOverlap :: () -> void #foreign imgui "?SetItemAllowOverlap@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    SetItemDefaultFocus :: () -> void #foreign imgui "_ZN5ImGui19SetItemDefaultFocusEv";
}

#if OS == .WINDOWS {
    SetItemDefaultFocus :: () -> void #foreign imgui "?SetItemDefaultFocus@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    SetKeyboardFocusHere :: (offset: s32 = 0) -> void #foreign imgui "_ZN5ImGui20SetKeyboardFocusHereEi";
}

#if OS == .WINDOWS {
    SetKeyboardFocusHere :: (offset: s32 = 0) -> void #foreign imgui "?SetKeyboardFocusHere@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    SetMouseCursor :: (cursor_type: MouseCursor) -> void #foreign imgui "_ZN5ImGui14SetMouseCursorEi";
}

#if OS == .WINDOWS {
    SetMouseCursor :: (cursor_type: MouseCursor) -> void #foreign imgui "?SetMouseCursor@ImGui@@YAXH@Z";
}

#if OS != .WINDOWS {
    SetNextFrameWantCaptureKeyboard :: (want_capture_keyboard: bool) -> void #foreign imgui "_ZN5ImGui31SetNextFrameWantCaptureKeyboardEb";
}

#if OS == .WINDOWS {
    SetNextFrameWantCaptureKeyboard :: (want_capture_keyboard: bool) -> void #foreign imgui "?SetNextFrameWantCaptureKeyboard@ImGui@@YAX_N@Z";
}

#if OS != .WINDOWS {
    SetNextFrameWantCaptureMouse :: (want_capture_mouse: bool) -> void #foreign imgui "_ZN5ImGui28SetNextFrameWantCaptureMouseEb";
}

#if OS == .WINDOWS {
    SetNextFrameWantCaptureMouse :: (want_capture_mouse: bool) -> void #foreign imgui "?SetNextFrameWantCaptureMouse@ImGui@@YAX_N@Z";
}

#if OS != .WINDOWS {
    SetNextItemOpen :: (is_open: bool, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui15SetNextItemOpenEbi";
}

#if OS == .WINDOWS {
    SetNextItemOpen :: (is_open: bool, cond: Cond = .None) -> void #foreign imgui "?SetNextItemOpen@ImGui@@YAX_NH@Z";
}

#if OS != .WINDOWS {
    SetNextItemWidth :: (item_width: float) -> void #foreign imgui "_ZN5ImGui16SetNextItemWidthEf";
}

#if OS == .WINDOWS {
    SetNextItemWidth :: (item_width: float) -> void #foreign imgui "?SetNextItemWidth@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetNextWindowBgAlpha :: (alpha: float) -> void #foreign imgui "_ZN5ImGui20SetNextWindowBgAlphaEf";
}

#if OS == .WINDOWS {
    SetNextWindowBgAlpha :: (alpha: float) -> void #foreign imgui "?SetNextWindowBgAlpha@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetNextWindowCollapsed :: (collapsed: bool, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui22SetNextWindowCollapsedEbi";
}

#if OS == .WINDOWS {
    SetNextWindowCollapsed :: (collapsed: bool, cond: Cond = .None) -> void #foreign imgui "?SetNextWindowCollapsed@ImGui@@YAX_NH@Z";
}

#if OS != .WINDOWS {
    SetNextWindowContentSize :: (size: *ImVec2) -> void #foreign imgui "_ZN5ImGui24SetNextWindowContentSizeERK6ImVec2";
}

#if OS == .WINDOWS {
    SetNextWindowContentSize :: (size: *ImVec2) -> void #foreign imgui "?SetNextWindowContentSize@ImGui@@YAXAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    SetNextWindowFocus :: () -> void #foreign imgui "_ZN5ImGui18SetNextWindowFocusEv";
}

#if OS == .WINDOWS {
    SetNextWindowFocus :: () -> void #foreign imgui "?SetNextWindowFocus@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    SetNextWindowPos :: (pos: *ImVec2, cond: Cond = .None, pivot: *ImVec2) -> void #foreign imgui "_ZN5ImGui16SetNextWindowPosERK6ImVec2iS2_";
}

#if OS == .WINDOWS {
    SetNextWindowPos :: (pos: *ImVec2, cond: Cond = .None, pivot: *ImVec2) -> void #foreign imgui "?SetNextWindowPos@ImGui@@YAXAEBUImVec2@@H0@Z";
}

#if OS != .WINDOWS {
    SetNextWindowScroll :: (scroll: *ImVec2) -> void #foreign imgui "_ZN5ImGui19SetNextWindowScrollERK6ImVec2";
}

#if OS == .WINDOWS {
    SetNextWindowScroll :: (scroll: *ImVec2) -> void #foreign imgui "?SetNextWindowScroll@ImGui@@YAXAEBUImVec2@@@Z";
}

#if OS != .WINDOWS {
    SetNextWindowSize :: (size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui17SetNextWindowSizeERK6ImVec2i";
}

#if OS == .WINDOWS {
    SetNextWindowSize :: (size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetNextWindowSize@ImGui@@YAXAEBUImVec2@@H@Z";
}

#if OS != .WINDOWS {
    SetNextWindowSizeConstraints :: (size_min: *ImVec2, size_max: *ImVec2, custom_callback: SizeCallback = null, custom_callback_data: *void = null) -> void #foreign imgui "_ZN5ImGui28SetNextWindowSizeConstraintsERK6ImVec2S2_PFvP21ImGuiSizeCallbackDataEPv";
}

#if OS == .WINDOWS {
    SetNextWindowSizeConstraints :: (size_min: *ImVec2, size_max: *ImVec2, custom_callback: SizeCallback = null, custom_callback_data: *void = null) -> void #foreign imgui "?SetNextWindowSizeConstraints@ImGui@@YAXAEBUImVec2@@0P6AXPEAUImGuiSizeCallbackData@@@ZPEAX@Z";
}

#if OS != .WINDOWS {
    SetScrollFromPosX :: (local_x: float, center_x_ratio: float = 0.5) -> void #foreign imgui "_ZN5ImGui17SetScrollFromPosXEff";
}

#if OS == .WINDOWS {
    SetScrollFromPosX :: (local_x: float, center_x_ratio: float = 0.5) -> void #foreign imgui "?SetScrollFromPosX@ImGui@@YAXMM@Z";
}

#if OS != .WINDOWS {
    SetScrollFromPosY :: (local_y: float, center_y_ratio: float = 0.5) -> void #foreign imgui "_ZN5ImGui17SetScrollFromPosYEff";
}

#if OS == .WINDOWS {
    SetScrollFromPosY :: (local_y: float, center_y_ratio: float = 0.5) -> void #foreign imgui "?SetScrollFromPosY@ImGui@@YAXMM@Z";
}

#if OS != .WINDOWS {
    SetScrollHereX :: (center_x_ratio: float = 0.5) -> void #foreign imgui "_ZN5ImGui14SetScrollHereXEf";
}

#if OS == .WINDOWS {
    SetScrollHereX :: (center_x_ratio: float = 0.5) -> void #foreign imgui "?SetScrollHereX@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetScrollHereY :: (center_y_ratio: float = 0.5) -> void #foreign imgui "_ZN5ImGui14SetScrollHereYEf";
}

#if OS == .WINDOWS {
    SetScrollHereY :: (center_y_ratio: float = 0.5) -> void #foreign imgui "?SetScrollHereY@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetScrollX :: (scroll_x: float) -> void #foreign imgui "_ZN5ImGui10SetScrollXEf";
}

#if OS == .WINDOWS {
    SetScrollX :: (scroll_x: float) -> void #foreign imgui "?SetScrollX@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetScrollY :: (scroll_y: float) -> void #foreign imgui "_ZN5ImGui10SetScrollYEf";
}

#if OS == .WINDOWS {
    SetScrollY :: (scroll_y: float) -> void #foreign imgui "?SetScrollY@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetStateStorage :: (storage: *Storage) -> void #foreign imgui "_ZN5ImGui15SetStateStorageEP12ImGuiStorage";
}

#if OS == .WINDOWS {
    SetStateStorage :: (storage: *Storage) -> void #foreign imgui "?SetStateStorage@ImGui@@YAXPEAUImGuiStorage@@@Z";
}

#if OS != .WINDOWS {
    SetTabItemClosed :: (tab_or_docked_window_label: *u8) -> void #foreign imgui "_ZN5ImGui16SetTabItemClosedEPKc";
}

#if OS == .WINDOWS {
    SetTabItemClosed :: (tab_or_docked_window_label: *u8) -> void #foreign imgui "?SetTabItemClosed@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    SetTooltip_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui10SetTooltipEPKcz";
}

#if OS == .WINDOWS {
    SetTooltip_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?SetTooltip@ImGui@@YAXPEBDZZ";
}

#if OS != .WINDOWS {
    SetWindowCollapsed :: (collapsed: bool, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui18SetWindowCollapsedEbi";
}

#if OS != .WINDOWS {
    SetWindowCollapsed :: (name: *u8, collapsed: bool, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui18SetWindowCollapsedEPKcbi";
}

#if OS == .WINDOWS {
    SetWindowCollapsed :: (collapsed: bool, cond: Cond = .None) -> void #foreign imgui "?SetWindowCollapsed@ImGui@@YAX_NH@Z";
}

#if OS == .WINDOWS {
    SetWindowCollapsed :: (name: *u8, collapsed: bool, cond: Cond = .None) -> void #foreign imgui "?SetWindowCollapsed@ImGui@@YAXPEBD_NH@Z";
}

#if OS != .WINDOWS {
    SetWindowFocus :: () -> void #foreign imgui "_ZN5ImGui14SetWindowFocusEv";
}

#if OS != .WINDOWS {
    SetWindowFocus :: (name: *u8) -> void #foreign imgui "_ZN5ImGui14SetWindowFocusEPKc";
}

#if OS == .WINDOWS {
    SetWindowFocus :: () -> void #foreign imgui "?SetWindowFocus@ImGui@@YAXXZ";
}

#if OS == .WINDOWS {
    SetWindowFocus :: (name: *u8) -> void #foreign imgui "?SetWindowFocus@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    SetWindowFontScale :: (scale: float) -> void #foreign imgui "_ZN5ImGui18SetWindowFontScaleEf";
}

#if OS == .WINDOWS {
    SetWindowFontScale :: (scale: float) -> void #foreign imgui "?SetWindowFontScale@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    SetWindowPos :: (pos: *ImVec2, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui12SetWindowPosERK6ImVec2i";
}

#if OS != .WINDOWS {
    SetWindowPos :: (name: *u8, pos: *ImVec2, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui12SetWindowPosEPKcRK6ImVec2i";
}

#if OS == .WINDOWS {
    SetWindowPos :: (pos: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowPos@ImGui@@YAXAEBUImVec2@@H@Z";
}

#if OS == .WINDOWS {
    SetWindowPos :: (name: *u8, pos: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowPos@ImGui@@YAXPEBDAEBUImVec2@@H@Z";
}

#if OS != .WINDOWS {
    SetWindowSize :: (size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui13SetWindowSizeERK6ImVec2i";
}

#if OS != .WINDOWS {
    SetWindowSize :: (name: *u8, size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "_ZN5ImGui13SetWindowSizeEPKcRK6ImVec2i";
}

#if OS == .WINDOWS {
    SetWindowSize :: (size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowSize@ImGui@@YAXAEBUImVec2@@H@Z";
}

#if OS == .WINDOWS {
    SetWindowSize :: (name: *u8, size: *ImVec2, cond: Cond = .None) -> void #foreign imgui "?SetWindowSize@ImGui@@YAXPEBDAEBUImVec2@@H@Z";
}

#if OS != .WINDOWS {
    ShowAboutWindow :: (p_open: *bool = null) -> void #foreign imgui "_ZN5ImGui15ShowAboutWindowEPb";
}

#if OS == .WINDOWS {
    ShowAboutWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowAboutWindow@ImGui@@YAXPEA_N@Z";
}

#if OS != .WINDOWS {
    ShowDebugLogWindow :: (p_open: *bool = null) -> void #foreign imgui "_ZN5ImGui18ShowDebugLogWindowEPb";
}

#if OS == .WINDOWS {
    ShowDebugLogWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowDebugLogWindow@ImGui@@YAXPEA_N@Z";
}

#if OS != .WINDOWS {
    ShowDemoWindow :: (p_open: *bool = null) -> void #foreign imgui "_ZN5ImGui14ShowDemoWindowEPb";
}

#if OS == .WINDOWS {
    ShowDemoWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowDemoWindow@ImGui@@YAXPEA_N@Z";
}

#if OS != .WINDOWS {
    ShowFontSelector :: (label: *u8) -> void #foreign imgui "_ZN5ImGui16ShowFontSelectorEPKc";
}

#if OS == .WINDOWS {
    ShowFontSelector :: (label: *u8) -> void #foreign imgui "?ShowFontSelector@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    ShowMetricsWindow :: (p_open: *bool = null) -> void #foreign imgui "_ZN5ImGui17ShowMetricsWindowEPb";
}

#if OS == .WINDOWS {
    ShowMetricsWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowMetricsWindow@ImGui@@YAXPEA_N@Z";
}

#if OS != .WINDOWS {
    ShowStackToolWindow :: (p_open: *bool = null) -> void #foreign imgui "_ZN5ImGui19ShowStackToolWindowEPb";
}

#if OS == .WINDOWS {
    ShowStackToolWindow :: (p_open: *bool = null) -> void #foreign imgui "?ShowStackToolWindow@ImGui@@YAXPEA_N@Z";
}

#if OS != .WINDOWS {
    ShowStyleEditor :: (ref: *Style = null) -> void #foreign imgui "_ZN5ImGui15ShowStyleEditorEP10ImGuiStyle";
}

#if OS == .WINDOWS {
    ShowStyleEditor :: (ref: *Style = null) -> void #foreign imgui "?ShowStyleEditor@ImGui@@YAXPEAUImGuiStyle@@@Z";
}

#if OS != .WINDOWS {
    ShowStyleSelector :: (label: *u8) -> bool #foreign imgui "_ZN5ImGui17ShowStyleSelectorEPKc";
}

#if OS == .WINDOWS {
    ShowStyleSelector :: (label: *u8) -> bool #foreign imgui "?ShowStyleSelector@ImGui@@YA_NPEBD@Z";
}

#if OS != .WINDOWS {
    ShowUserGuide :: () -> void #foreign imgui "_ZN5ImGui13ShowUserGuideEv";
}

#if OS == .WINDOWS {
    ShowUserGuide :: () -> void #foreign imgui "?ShowUserGuide@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    SliderAngle :: (label: *u8, v_rad: *float, v_degrees_min: float = -360.0, v_degrees_max: float = +360.0, format: *u8 = "%.0f deg", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui11SliderAngleEPKcPfffS1_i";
}

#if OS == .WINDOWS {
    SliderAngle :: (label: *u8, v_rad: *float, v_degrees_min: float = -360.0, v_degrees_max: float = +360.0, format: *u8 = "%.0f deg", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderAngle@ImGui@@YA_NPEBDPEAMMM0H@Z";
}

#if OS != .WINDOWS {
    SliderFloat :: (label: *u8, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui11SliderFloatEPKcPfffS1_i";
}

#if OS == .WINDOWS {
    SliderFloat :: (label: *u8, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat@ImGui@@YA_NPEBDPEAMMM0H@Z";
}

#if OS != .WINDOWS {
    SliderFloat2 :: (label: *u8, v: *[2] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui12SliderFloat2EPKcPfffS1_i";
}

#if OS == .WINDOWS {
    SliderFloat2 :: (label: *u8, v: *[2] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat2@ImGui@@YA_NPEBDQEAMMM0H@Z";
}

#if OS != .WINDOWS {
    SliderFloat3 :: (label: *u8, v: *[3] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui12SliderFloat3EPKcPfffS1_i";
}

#if OS == .WINDOWS {
    SliderFloat3 :: (label: *u8, v: *[3] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat3@ImGui@@YA_NPEBDQEAMMM0H@Z";
}

#if OS != .WINDOWS {
    SliderFloat4 :: (label: *u8, v: *[4] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui12SliderFloat4EPKcPfffS1_i";
}

#if OS == .WINDOWS {
    SliderFloat4 :: (label: *u8, v: *[4] float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderFloat4@ImGui@@YA_NPEBDQEAMMM0H@Z";
}

#if OS != .WINDOWS {
    SliderInt :: (label: *u8, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui9SliderIntEPKcPiiiS1_i";
}

#if OS == .WINDOWS {
    SliderInt :: (label: *u8, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt@ImGui@@YA_NPEBDPEAHHH0H@Z";
}

#if OS != .WINDOWS {
    SliderInt2 :: (label: *u8, v: *[2] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10SliderInt2EPKcPiiiS1_i";
}

#if OS == .WINDOWS {
    SliderInt2 :: (label: *u8, v: *[2] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt2@ImGui@@YA_NPEBDQEAHHH0H@Z";
}

#if OS != .WINDOWS {
    SliderInt3 :: (label: *u8, v: *[3] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10SliderInt3EPKcPiiiS1_i";
}

#if OS == .WINDOWS {
    SliderInt3 :: (label: *u8, v: *[3] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt3@ImGui@@YA_NPEBDQEAHHH0H@Z";
}

#if OS != .WINDOWS {
    SliderInt4 :: (label: *u8, v: *[4] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10SliderInt4EPKcPiiiS1_i";
}

#if OS == .WINDOWS {
    SliderInt4 :: (label: *u8, v: *[4] s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?SliderInt4@ImGui@@YA_NPEBDQEAHHH0H@Z";
}

#if OS != .WINDOWS {
    SliderScalar :: (label: *u8, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui12SliderScalarEPKciPvPKvS4_S1_i";
}

#if OS == .WINDOWS {
    SliderScalar :: (label: *u8, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?SliderScalar@ImGui@@YA_NPEBDHPEAXPEBX20H@Z";
}

#if OS != .WINDOWS {
    SliderScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui13SliderScalarNEPKciPviPKvS4_S1_i";
}

#if OS == .WINDOWS {
    SliderScalarN :: (label: *u8, data_type: DataType, p_data: *void, components: s32, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?SliderScalarN@ImGui@@YA_NPEBDHPEAXHPEBX20H@Z";
}

#if OS != .WINDOWS {
    SmallButton :: (label: *u8) -> bool #foreign imgui "_ZN5ImGui11SmallButtonEPKc";
}

#if OS == .WINDOWS {
    SmallButton :: (label: *u8) -> bool #foreign imgui "?SmallButton@ImGui@@YA_NPEBD@Z";
}

#if OS != .WINDOWS {
    Spacing :: () -> void #foreign imgui "_ZN5ImGui7SpacingEv";
}

#if OS == .WINDOWS {
    Spacing :: () -> void #foreign imgui "?Spacing@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    StyleColorsClassic :: (dst: *Style = null) -> void #foreign imgui "_ZN5ImGui18StyleColorsClassicEP10ImGuiStyle";
}

#if OS == .WINDOWS {
    StyleColorsClassic :: (dst: *Style = null) -> void #foreign imgui "?StyleColorsClassic@ImGui@@YAXPEAUImGuiStyle@@@Z";
}

#if OS != .WINDOWS {
    StyleColorsDark :: (dst: *Style = null) -> void #foreign imgui "_ZN5ImGui15StyleColorsDarkEP10ImGuiStyle";
}

#if OS == .WINDOWS {
    StyleColorsDark :: (dst: *Style = null) -> void #foreign imgui "?StyleColorsDark@ImGui@@YAXPEAUImGuiStyle@@@Z";
}

#if OS != .WINDOWS {
    StyleColorsLight :: (dst: *Style = null) -> void #foreign imgui "_ZN5ImGui16StyleColorsLightEP10ImGuiStyle";
}

#if OS == .WINDOWS {
    StyleColorsLight :: (dst: *Style = null) -> void #foreign imgui "?StyleColorsLight@ImGui@@YAXPEAUImGuiStyle@@@Z";
}

#if OS != .WINDOWS {
    TabItemButton :: (label: *u8, flags: TabItemFlags = .None) -> bool #foreign imgui "_ZN5ImGui13TabItemButtonEPKci";
}

#if OS == .WINDOWS {
    TabItemButton :: (label: *u8, flags: TabItemFlags = .None) -> bool #foreign imgui "?TabItemButton@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    TableGetColumnCount :: () -> s32 #foreign imgui "_ZN5ImGui19TableGetColumnCountEv";
}

#if OS == .WINDOWS {
    TableGetColumnCount :: () -> s32 #foreign imgui "?TableGetColumnCount@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    TableGetColumnFlags :: (column_n: s32 = -1) -> TableColumnFlags #foreign imgui "_ZN5ImGui19TableGetColumnFlagsEi";
}

#if OS == .WINDOWS {
    TableGetColumnFlags :: (column_n: s32 = -1) -> TableColumnFlags #foreign imgui "?TableGetColumnFlags@ImGui@@YAHH@Z";
}

#if OS != .WINDOWS {
    TableGetColumnIndex :: () -> s32 #foreign imgui "_ZN5ImGui19TableGetColumnIndexEv";
}

#if OS == .WINDOWS {
    TableGetColumnIndex :: () -> s32 #foreign imgui "?TableGetColumnIndex@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    TableGetColumnName :: (column_n: s32 = -1) -> *u8 #foreign imgui "_ZN5ImGui18TableGetColumnNameEi";
}

#if OS == .WINDOWS {
    TableGetColumnName :: (column_n: s32 = -1) -> *u8 #foreign imgui "?TableGetColumnName@ImGui@@YAPEBDH@Z";
}

#if OS != .WINDOWS {
    TableGetRowIndex :: () -> s32 #foreign imgui "_ZN5ImGui16TableGetRowIndexEv";
}

#if OS == .WINDOWS {
    TableGetRowIndex :: () -> s32 #foreign imgui "?TableGetRowIndex@ImGui@@YAHXZ";
}

#if OS != .WINDOWS {
    TableGetSortSpecs :: () -> *TableSortSpecs #foreign imgui "_ZN5ImGui17TableGetSortSpecsEv";
}

#if OS == .WINDOWS {
    TableGetSortSpecs :: () -> *TableSortSpecs #foreign imgui "?TableGetSortSpecs@ImGui@@YAPEAUImGuiTableSortSpecs@@XZ";
}

#if OS != .WINDOWS {
    TableHeader :: (label: *u8) -> void #foreign imgui "_ZN5ImGui11TableHeaderEPKc";
}

#if OS == .WINDOWS {
    TableHeader :: (label: *u8) -> void #foreign imgui "?TableHeader@ImGui@@YAXPEBD@Z";
}

#if OS != .WINDOWS {
    TableHeadersRow :: () -> void #foreign imgui "_ZN5ImGui15TableHeadersRowEv";
}

#if OS == .WINDOWS {
    TableHeadersRow :: () -> void #foreign imgui "?TableHeadersRow@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    TableNextColumn :: () -> bool #foreign imgui "_ZN5ImGui15TableNextColumnEv";
}

#if OS == .WINDOWS {
    TableNextColumn :: () -> bool #foreign imgui "?TableNextColumn@ImGui@@YA_NXZ";
}

#if OS != .WINDOWS {
    TableNextRow :: (row_flags: TableRowFlags = .None, min_row_height: float = 0.0) -> void #foreign imgui "_ZN5ImGui12TableNextRowEif";
}

#if OS == .WINDOWS {
    TableNextRow :: (row_flags: TableRowFlags = .None, min_row_height: float = 0.0) -> void #foreign imgui "?TableNextRow@ImGui@@YAXHM@Z";
}

#if OS != .WINDOWS {
    TableSetBgColor :: (target: TableBgTarget, color: ImU32, column_n: s32 = -1) -> void #foreign imgui "_ZN5ImGui15TableSetBgColorEiji";
}

#if OS == .WINDOWS {
    TableSetBgColor :: (target: TableBgTarget, color: ImU32, column_n: s32 = -1) -> void #foreign imgui "?TableSetBgColor@ImGui@@YAXHIH@Z";
}

#if OS != .WINDOWS {
    TableSetColumnEnabled :: (column_n: s32, v: bool) -> void #foreign imgui "_ZN5ImGui21TableSetColumnEnabledEib";
}

#if OS == .WINDOWS {
    TableSetColumnEnabled :: (column_n: s32, v: bool) -> void #foreign imgui "?TableSetColumnEnabled@ImGui@@YAXH_N@Z";
}

#if OS != .WINDOWS {
    TableSetColumnIndex :: (column_n: s32) -> bool #foreign imgui "_ZN5ImGui19TableSetColumnIndexEi";
}

#if OS == .WINDOWS {
    TableSetColumnIndex :: (column_n: s32) -> bool #foreign imgui "?TableSetColumnIndex@ImGui@@YA_NH@Z";
}

#if OS != .WINDOWS {
    TableSetupColumn :: (label: *u8, flags: TableColumnFlags = .None, init_width_or_weight: float = 0.0, user_id: ID = 0) -> void #foreign imgui "_ZN5ImGui16TableSetupColumnEPKcifj";
}

#if OS == .WINDOWS {
    TableSetupColumn :: (label: *u8, flags: TableColumnFlags = .None, init_width_or_weight: float = 0.0, user_id: ID = 0) -> void #foreign imgui "?TableSetupColumn@ImGui@@YAXPEBDHMI@Z";
}

#if OS != .WINDOWS {
    TableSetupScrollFreeze :: (cols: s32, rows: s32) -> void #foreign imgui "_ZN5ImGui22TableSetupScrollFreezeEii";
}

#if OS == .WINDOWS {
    TableSetupScrollFreeze :: (cols: s32, rows: s32) -> void #foreign imgui "?TableSetupScrollFreeze@ImGui@@YAXHH@Z";
}

#if OS != .WINDOWS {
    Text_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui4TextEPKcz";
}

#if OS == .WINDOWS {
    Text_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?Text@ImGui@@YAXPEBDZZ";
}

#if OS != .WINDOWS {
    TextColored_CFormat :: (col: *ImVec4, fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui11TextColoredERK6ImVec4PKcz";
}

#if OS == .WINDOWS {
    TextColored_CFormat :: (col: *ImVec4, fmt: *u8, __args: ..Any) -> void #foreign imgui "?TextColored@ImGui@@YAXAEBUImVec4@@PEBDZZ";
}

#if OS != .WINDOWS {
    TextDisabled_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui12TextDisabledEPKcz";
}

#if OS == .WINDOWS {
    TextDisabled_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?TextDisabled@ImGui@@YAXPEBDZZ";
}

#if OS != .WINDOWS {
    TextUnformatted :: (text: *u8, text_end: *u8 = null) -> void #foreign imgui "_ZN5ImGui15TextUnformattedEPKcS1_";
}

#if OS == .WINDOWS {
    TextUnformatted :: (text: *u8, text_end: *u8 = null) -> void #foreign imgui "?TextUnformatted@ImGui@@YAXPEBD0@Z";
}

#if OS != .WINDOWS {
    TextWrapped_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "_ZN5ImGui11TextWrappedEPKcz";
}

#if OS == .WINDOWS {
    TextWrapped_CFormat :: (fmt: *u8, __args: ..Any) -> void #foreign imgui "?TextWrapped@ImGui@@YAXPEBDZZ";
}

#if OS != .WINDOWS {
    TreeNode :: (label: *u8) -> bool #foreign imgui "_ZN5ImGui8TreeNodeEPKc";
}

#if OS == .WINDOWS {
    TreeNode :: (label: *u8) -> bool #foreign imgui "?TreeNode@ImGui@@YA_NPEBD@Z";
}

#if OS != .WINDOWS {
    TreeNode_CFormat :: (str_id: *u8, fmt: *u8, __args: ..Any) -> bool #foreign imgui "_ZN5ImGui8TreeNodeEPKcS1_z";
}

#if OS != .WINDOWS {
    TreeNode_CFormat :: (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #foreign imgui "_ZN5ImGui8TreeNodeEPKvPKcz";
}

#if OS == .WINDOWS {
    TreeNode_CFormat :: (str_id: *u8, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNode@ImGui@@YA_NPEBD0ZZ";
}

#if OS == .WINDOWS {
    TreeNode_CFormat :: (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNode@ImGui@@YA_NPEBXPEBDZZ";
}

#if OS != .WINDOWS {
    TreeNodeEx :: (label: *u8, flags: TreeNodeFlags = .None) -> bool #foreign imgui "_ZN5ImGui10TreeNodeExEPKci";
}

#if OS == .WINDOWS {
    TreeNodeEx :: (label: *u8, flags: TreeNodeFlags = .None) -> bool #foreign imgui "?TreeNodeEx@ImGui@@YA_NPEBDH@Z";
}

#if OS != .WINDOWS {
    TreeNodeEx_CFormat :: (str_id: *u8, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #foreign imgui "_ZN5ImGui10TreeNodeExEPKciS1_z";
}

#if OS != .WINDOWS {
    TreeNodeEx_CFormat :: (ptr_id: *void, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #foreign imgui "_ZN5ImGui10TreeNodeExEPKviPKcz";
}

#if OS == .WINDOWS {
    TreeNodeEx_CFormat :: (str_id: *u8, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNodeEx@ImGui@@YA_NPEBDH0ZZ";
}

#if OS == .WINDOWS {
    TreeNodeEx_CFormat :: (ptr_id: *void, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #foreign imgui "?TreeNodeEx@ImGui@@YA_NPEBXHPEBDZZ";
}

#if OS != .WINDOWS {
    TreePop :: () -> void #foreign imgui "_ZN5ImGui7TreePopEv";
}

#if OS == .WINDOWS {
    TreePop :: () -> void #foreign imgui "?TreePop@ImGui@@YAXXZ";
}

#if OS != .WINDOWS {
    TreePush :: (str_id: *u8) -> void #foreign imgui "_ZN5ImGui8TreePushEPKc";
}

#if OS != .WINDOWS {
    TreePush :: (ptr_id: *void) -> void #foreign imgui "_ZN5ImGui8TreePushEPKv";
}

#if OS == .WINDOWS {
    TreePush :: (str_id: *u8) -> void #foreign imgui "?TreePush@ImGui@@YAXPEBD@Z";
}

#if OS == .WINDOWS {
    TreePush :: (ptr_id: *void) -> void #foreign imgui "?TreePush@ImGui@@YAXPEBX@Z";
}

#if OS != .WINDOWS {
    Unindent :: (indent_w: float = 0.0) -> void #foreign imgui "_ZN5ImGui8UnindentEf";
}

#if OS == .WINDOWS {
    Unindent :: (indent_w: float = 0.0) -> void #foreign imgui "?Unindent@ImGui@@YAXM@Z";
}

#if OS != .WINDOWS {
    Value :: (prefix: *u8, b: bool) -> void #foreign imgui "_ZN5ImGui5ValueEPKcb";
}

#if OS != .WINDOWS {
    Value :: (prefix: *u8, v: s32) -> void #foreign imgui "_ZN5ImGui5ValueEPKci";
}

#if OS != .WINDOWS {
    Value :: (prefix: *u8, v: u32) -> void #foreign imgui "_ZN5ImGui5ValueEPKcj";
}

#if OS != .WINDOWS {
    Value :: (prefix: *u8, v: float, float_format: *u8 = null) -> void #foreign imgui "_ZN5ImGui5ValueEPKcfS1_";
}

#if OS == .WINDOWS {
    Value :: (prefix: *u8, b: bool) -> void #foreign imgui "?Value@ImGui@@YAXPEBD_N@Z";
}

#if OS == .WINDOWS {
    Value :: (prefix: *u8, v: s32) -> void #foreign imgui "?Value@ImGui@@YAXPEBDH@Z";
}

#if OS == .WINDOWS {
    Value :: (prefix: *u8, v: u32) -> void #foreign imgui "?Value@ImGui@@YAXPEBDI@Z";
}

#if OS == .WINDOWS {
    Value :: (prefix: *u8, v: float, float_format: *u8 = null) -> void #foreign imgui "?Value@ImGui@@YAXPEBDM0@Z";
}

#if OS != .WINDOWS {
    VSliderFloat :: (label: *u8, size: *ImVec2, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui12VSliderFloatEPKcRK6ImVec2PfffS1_i";
}

#if OS == .WINDOWS {
    VSliderFloat :: (label: *u8, size: *ImVec2, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #foreign imgui "?VSliderFloat@ImGui@@YA_NPEBDAEBUImVec2@@PEAMMM0H@Z";
}

#if OS != .WINDOWS {
    VSliderInt :: (label: *u8, size: *ImVec2, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui10VSliderIntEPKcRK6ImVec2PiiiS1_i";
}

#if OS == .WINDOWS {
    VSliderInt :: (label: *u8, size: *ImVec2, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #foreign imgui "?VSliderInt@ImGui@@YA_NPEBDAEBUImVec2@@PEAHHH0H@Z";
}

#if OS != .WINDOWS {
    VSliderScalar :: (label: *u8, size: *ImVec2, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "_ZN5ImGui13VSliderScalarEPKcRK6ImVec2iPvPKvS7_S1_i";
}

#if OS == .WINDOWS {
    VSliderScalar :: (label: *u8, size: *ImVec2, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #foreign imgui "?VSliderScalar@ImGui@@YA_NPEBDAEBUImVec2@@HPEAXPEBX30H@Z";
}

// Convenience operations

#if OS != .WINDOWS {
    BeginChild :: (str_id: *u8, size: ImVec2 = ImVec2.{ 0, 0 }, border := false, flags: WindowFlags = .None) -> bool #no_context {
        return BeginChild(str_id, *size, border, flags);
    }
}

#if OS != .WINDOWS {
    BeginChild :: (id: ID, size: ImVec2 = ImVec2.{ 0, 0 }, border := false, flags: WindowFlags = .None) -> bool #no_context {
        return BeginChild(id, *size, border, flags);
    }
}

#if OS == .WINDOWS {
    BeginChild :: (str_id: *u8, size: ImVec2 = ImVec2.{ 0, 0 }, border := false, flags: WindowFlags = .None) -> bool #no_context {
        return BeginChild(str_id, *size, border, flags);
    }
}

#if OS == .WINDOWS {
    BeginChild :: (id: ID, size: ImVec2 = ImVec2.{ 0, 0 }, border := false, flags: WindowFlags = .None) -> bool #no_context {
        return BeginChild(id, *size, border, flags);
    }
}

#if OS != .WINDOWS {
    BeginChildFrame :: (id: ID, size: ImVec2, flags: WindowFlags = .None) -> bool #no_context {
        return BeginChildFrame(id, *size, flags);
    }
}

#if OS == .WINDOWS {
    BeginChildFrame :: (id: ID, size: ImVec2, flags: WindowFlags = .None) -> bool #no_context {
        return BeginChildFrame(id, *size, flags);
    }
}

#if OS != .WINDOWS {
    BeginListBox :: (label: *u8, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return BeginListBox(label, *size);
    }
}

#if OS == .WINDOWS {
    BeginListBox :: (label: *u8, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return BeginListBox(label, *size);
    }
}

#if OS != .WINDOWS {
    BeginTable :: (str_id: *u8, column: s32, flags: TableFlags = .None, outer_size: ImVec2 = ImVec2.{ 0.0, 0.0 }, inner_width: float = 0.0) -> bool #no_context {
        return BeginTable(str_id, column, flags, *outer_size, inner_width);
    }
}

#if OS == .WINDOWS {
    BeginTable :: (str_id: *u8, column: s32, flags: TableFlags = .None, outer_size: ImVec2 = ImVec2.{ 0.0, 0.0 }, inner_width: float = 0.0) -> bool #no_context {
        return BeginTable(str_id, column, flags, *outer_size, inner_width);
    }
}

#if OS != .WINDOWS {
    BulletText :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = BulletText_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    BulletText :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = BulletText_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    Button :: (label: *u8, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return Button(label, *size);
    }
}

#if OS == .WINDOWS {
    Button :: (label: *u8, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return Button(label, *size);
    }
}

#if OS != .WINDOWS {
    CalcTextSize :: (text: string, hide_text_after_double_hash := false, wrap_width: float = -1.0) -> ImVec2 #no_context {
        return CalcTextSize(text.data, text.data + text.count, hide_text_after_double_hash, wrap_width);
    }
}

#if OS == .WINDOWS {
    CalcTextSize :: (text: string, hide_text_after_double_hash := false, wrap_width: float = -1.0) -> ImVec2 #no_context {
        return CalcTextSize(text.data, text.data + text.count, hide_text_after_double_hash, wrap_width);
    }
}

#if OS != .WINDOWS {
    ColorButton :: (desc_id: *u8, col: ImVec4, flags: ColorEditFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return ColorButton(desc_id, *col, flags, *size);
    }
}

#if OS == .WINDOWS {
    ColorButton :: (desc_id: *u8, col: ImVec4, flags: ColorEditFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return ColorButton(desc_id, *col, flags, *size);
    }
}

#if OS != .WINDOWS {
    ColorConvertFloat4ToU32 :: (in: ImVec4) -> ImU32 #no_context {
        return ColorConvertFloat4ToU32(*in);
    }
}

#if OS == .WINDOWS {
    ColorConvertFloat4ToU32 :: (in: ImVec4) -> ImU32 #no_context {
        return ColorConvertFloat4ToU32(*in);
    }
}

#if OS != .WINDOWS {
    Dummy :: (size: ImVec2) #no_context {
        Dummy(*size);
    }
}

#if OS == .WINDOWS {
    Dummy :: (size: ImVec2) #no_context {
        Dummy(*size);
    }
}

#if OS != .WINDOWS {
    get_Codepoint :: (fields: ImFontGlyph) -> u32 {
        return cast(u32) ((fields.__bitfield >> 2) & 1073741823);
    }
}

#if OS == .WINDOWS {
    get_Codepoint :: (fields: ImFontGlyph) -> u32 {
        return cast(u32) ((fields.__bitfield >> 2) & 1073741823);
    }
}

#if OS != .WINDOWS {
    get_Colored :: (fields: ImFontGlyph) -> bool {
        return ((fields.__bitfield >> 0) & 1) != 0;
    }
}

#if OS == .WINDOWS {
    get_Colored :: (fields: ImFontGlyph) -> bool {
        return ((fields.__bitfield >> 0) & 1) != 0;
    }
}

#if OS != .WINDOWS {
    get_Visible :: (fields: ImFontGlyph) -> bool {
        return ((fields.__bitfield >> 1) & 1) != 0;
    }
}

#if OS == .WINDOWS {
    get_Visible :: (fields: ImFontGlyph) -> bool {
        return ((fields.__bitfield >> 1) & 1) != 0;
    }
}

#if OS != .WINDOWS {
    GetColorU32 :: (col: ImVec4) -> ImU32 #no_context {
        return GetColorU32(*col);
    }
}

#if OS == .WINDOWS {
    GetColorU32 :: (col: ImVec4) -> ImU32 #no_context {
        return GetColorU32(*col);
    }
}

#if OS != .WINDOWS {
    GetID :: (str_id: string) -> ID #no_context {
        return GetID(str_id.data, str_id.data + str_id.count);
    }
}

#if OS == .WINDOWS {
    GetID :: (str_id: string) -> ID #no_context {
        return GetID(str_id.data, str_id.data + str_id.count);
    }
}

#if OS != .WINDOWS {
    Image :: (user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }, border_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }) #no_context {
        Image(user_texture_id, *size, *uv0, *uv1, *tint_col, *border_col);
    }
}

#if OS == .WINDOWS {
    Image :: (user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }, border_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }) #no_context {
        Image(user_texture_id, *size, *uv0, *uv1, *tint_col, *border_col);
    }
}

#if OS != .WINDOWS {
    ImageButton :: (str_id: *u8, user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, bg_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }) -> bool #no_context {
        return ImageButton(str_id, user_texture_id, *size, *uv0, *uv1, *bg_col, *tint_col);
    }
}

#if OS != .WINDOWS {
    ImageButton :: (user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, frame_padding: s32 = -1, bg_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }) -> bool #no_context {
        return ImageButton(user_texture_id, *size, *uv0, *uv1, frame_padding, *bg_col, *tint_col);
    }
}

#if OS == .WINDOWS {
    ImageButton :: (str_id: *u8, user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, bg_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }) -> bool #no_context {
        return ImageButton(str_id, user_texture_id, *size, *uv0, *uv1, *bg_col, *tint_col);
    }
}

#if OS == .WINDOWS {
    ImageButton :: (user_texture_id: ImTextureID, size: ImVec2, uv0: ImVec2 = ImVec2.{ 0, 0 }, uv1: ImVec2 = ImVec2.{ 1, 1 }, frame_padding: s32 = -1, bg_col: ImVec4 = ImVec4.{ 0, 0, 0, 0 }, tint_col: ImVec4 = ImVec4.{ 1, 1, 1, 1 }) -> bool #no_context {
        return ImageButton(user_texture_id, *size, *uv0, *uv1, frame_padding, *bg_col, *tint_col);
    }
}

#if OS != .WINDOWS {
    InputTextMultiline :: (label: *u8, buf: *u8, buf_size: u64, size: ImVec2 = ImVec2.{ 0, 0 }, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #no_context {
        return InputTextMultiline(label, buf, buf_size, *size, flags, callback, user_data);
    }
}

#if OS == .WINDOWS {
    InputTextMultiline :: (label: *u8, buf: *u8, buf_size: u64, size: ImVec2 = ImVec2.{ 0, 0 }, flags: InputTextFlags = .None, callback: InputTextCallback = null, user_data: *void = null) -> bool #no_context {
        return InputTextMultiline(label, buf, buf_size, *size, flags, callback, user_data);
    }
}

#if OS != .WINDOWS {
    InvisibleButton :: (str_id: *u8, size: ImVec2, flags: ButtonFlags = .None) -> bool #no_context {
        return InvisibleButton(str_id, *size, flags);
    }
}

#if OS == .WINDOWS {
    InvisibleButton :: (str_id: *u8, size: ImVec2, flags: ButtonFlags = .None) -> bool #no_context {
        return InvisibleButton(str_id, *size, flags);
    }
}

#if OS != .WINDOWS {
    IsMouseHoveringRect :: (r_min: ImVec2, r_max: ImVec2, clip := true) -> bool #no_context {
        return IsMouseHoveringRect(*r_min, *r_max, clip);
    }
}

#if OS == .WINDOWS {
    IsMouseHoveringRect :: (r_min: ImVec2, r_max: ImVec2, clip := true) -> bool #no_context {
        return IsMouseHoveringRect(*r_min, *r_max, clip);
    }
}

#if OS != .WINDOWS {
    IsRectVisible :: (size: ImVec2) -> bool #no_context {
        return IsRectVisible(*size);
    }
}

#if OS != .WINDOWS {
    IsRectVisible :: (rect_min: ImVec2, rect_max: ImVec2) -> bool #no_context {
        return IsRectVisible(*rect_min, *rect_max);
    }
}

#if OS == .WINDOWS {
    IsRectVisible :: (size: ImVec2) -> bool #no_context {
        return IsRectVisible(*size);
    }
}

#if OS == .WINDOWS {
    IsRectVisible :: (rect_min: ImVec2, rect_max: ImVec2) -> bool #no_context {
        return IsRectVisible(*rect_min, *rect_max);
    }
}

#if OS != .WINDOWS {
    LabelText :: (label: *u8, fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) label }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (label: *u8, fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = LabelText_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    LabelText :: (label: *u8, fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) label }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (label: *u8, fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = LabelText_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    LogText :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = LogText_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    LogText :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = LogText_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    ProgressBar :: (fraction: float, size_arg: ImVec2 = ImVec2.{ -FLOAT32_MIN, 0 }, overlay: *u8 = null) #no_context {
        ProgressBar(fraction, *size_arg, overlay);
    }
}

#if OS == .WINDOWS {
    ProgressBar :: (fraction: float, size_arg: ImVec2 = ImVec2.{ -FLOAT32_MIN, 0 }, overlay: *u8 = null) #no_context {
        ProgressBar(fraction, *size_arg, overlay);
    }
}

#if OS != .WINDOWS {
    PushClipRect :: (clip_rect_min: ImVec2, clip_rect_max: ImVec2, intersect_with_current_clip_rect: bool) #no_context {
        PushClipRect(*clip_rect_min, *clip_rect_max, intersect_with_current_clip_rect);
    }
}

#if OS == .WINDOWS {
    PushClipRect :: (clip_rect_min: ImVec2, clip_rect_max: ImVec2, intersect_with_current_clip_rect: bool) #no_context {
        PushClipRect(*clip_rect_min, *clip_rect_max, intersect_with_current_clip_rect);
    }
}

#if OS != .WINDOWS {
    PushID :: (str_id: string) #no_context {
        PushID(str_id.data, str_id.data + str_id.count);
    }
}

#if OS == .WINDOWS {
    PushID :: (str_id: string) #no_context {
        PushID(str_id.data, str_id.data + str_id.count);
    }
}

#if OS != .WINDOWS {
    PushStyleColor :: (idx: Col, col: ImVec4) #no_context {
        PushStyleColor(idx, *col);
    }
}

#if OS == .WINDOWS {
    PushStyleColor :: (idx: Col, col: ImVec4) #no_context {
        PushStyleColor(idx, *col);
    }
}

#if OS != .WINDOWS {
    PushStyleVar :: (idx: StyleVar, val: ImVec2) #no_context {
        PushStyleVar(idx, *val);
    }
}

#if OS == .WINDOWS {
    PushStyleVar :: (idx: StyleVar, val: ImVec2) #no_context {
        PushStyleVar(idx, *val);
    }
}

#if OS != .WINDOWS {
    Selectable :: (label: *u8, selected := false, flags: SelectableFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return Selectable(label, selected, flags, *size);
    }
}

#if OS != .WINDOWS {
    Selectable :: (label: *u8, p_selected: *bool, flags: SelectableFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return Selectable(label, p_selected, flags, *size);
    }
}

#if OS == .WINDOWS {
    Selectable :: (label: *u8, selected := false, flags: SelectableFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return Selectable(label, selected, flags, *size);
    }
}

#if OS == .WINDOWS {
    Selectable :: (label: *u8, p_selected: *bool, flags: SelectableFlags = .None, size: ImVec2 = ImVec2.{ 0, 0 }) -> bool #no_context {
        return Selectable(label, p_selected, flags, *size);
    }
}

#if OS != .WINDOWS {
    set_Codepoint :: (fields: *ImFontGlyph, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4294967292)) | ((value & 1073741823) << 2);
    }
}

#if OS == .WINDOWS {
    set_Codepoint :: (fields: *ImFontGlyph, value: u32) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 4294967292)) | ((value & 1073741823) << 2);
    }
}

#if OS != .WINDOWS {
    set_Colored :: (fields: *ImFontGlyph, value: bool) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 1)) | (((cast(u32) value) & 1) << 0);
    }
}

#if OS == .WINDOWS {
    set_Colored :: (fields: *ImFontGlyph, value: bool) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 1)) | (((cast(u32) value) & 1) << 0);
    }
}

#if OS != .WINDOWS {
    set_Visible :: (fields: *ImFontGlyph, value: bool) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 2)) | (((cast(u32) value) & 1) << 1);
    }
}

#if OS == .WINDOWS {
    set_Visible :: (fields: *ImFontGlyph, value: bool) {
        fields.__bitfield = (fields.__bitfield & ~(cast(u32) 2)) | (((cast(u32) value) & 1) << 1);
    }
}

#if OS != .WINDOWS {
    SetCursorPos :: (local_pos: ImVec2) #no_context {
        SetCursorPos(*local_pos);
    }
}

#if OS == .WINDOWS {
    SetCursorPos :: (local_pos: ImVec2) #no_context {
        SetCursorPos(*local_pos);
    }
}

#if OS != .WINDOWS {
    SetCursorScreenPos :: (pos: ImVec2) #no_context {
        SetCursorScreenPos(*pos);
    }
}

#if OS == .WINDOWS {
    SetCursorScreenPos :: (pos: ImVec2) #no_context {
        SetCursorScreenPos(*pos);
    }
}

#if OS != .WINDOWS {
    SetNextWindowContentSize :: (size: ImVec2) #no_context {
        SetNextWindowContentSize(*size);
    }
}

#if OS == .WINDOWS {
    SetNextWindowContentSize :: (size: ImVec2) #no_context {
        SetNextWindowContentSize(*size);
    }
}

#if OS != .WINDOWS {
    SetNextWindowPos :: (pos: ImVec2, cond: Cond = .None, pivot: ImVec2 = ImVec2.{ 0, 0 }) #no_context {
        SetNextWindowPos(*pos, cond, *pivot);
    }
}

#if OS == .WINDOWS {
    SetNextWindowPos :: (pos: ImVec2, cond: Cond = .None, pivot: ImVec2 = ImVec2.{ 0, 0 }) #no_context {
        SetNextWindowPos(*pos, cond, *pivot);
    }
}

#if OS != .WINDOWS {
    SetNextWindowScroll :: (scroll: ImVec2) #no_context {
        SetNextWindowScroll(*scroll);
    }
}

#if OS == .WINDOWS {
    SetNextWindowScroll :: (scroll: ImVec2) #no_context {
        SetNextWindowScroll(*scroll);
    }
}

#if OS != .WINDOWS {
    SetNextWindowSize :: (size: ImVec2, cond: Cond = .None) #no_context {
        SetNextWindowSize(*size, cond);
    }
}

#if OS == .WINDOWS {
    SetNextWindowSize :: (size: ImVec2, cond: Cond = .None) #no_context {
        SetNextWindowSize(*size, cond);
    }
}

#if OS != .WINDOWS {
    SetNextWindowSizeConstraints :: (size_min: ImVec2, size_max: ImVec2, custom_callback: SizeCallback = null, custom_callback_data: *void = null) #no_context {
        SetNextWindowSizeConstraints(*size_min, *size_max, custom_callback, custom_callback_data);
    }
}

#if OS == .WINDOWS {
    SetNextWindowSizeConstraints :: (size_min: ImVec2, size_max: ImVec2, custom_callback: SizeCallback = null, custom_callback_data: *void = null) #no_context {
        SetNextWindowSizeConstraints(*size_min, *size_max, custom_callback, custom_callback_data);
    }
}

#if OS != .WINDOWS {
    SetTooltip :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = SetTooltip_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    SetTooltip :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = SetTooltip_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    SetWindowPos :: (pos: ImVec2, cond: Cond = .None) #no_context {
        SetWindowPos(*pos, cond);
    }
}

#if OS != .WINDOWS {
    SetWindowPos :: (name: *u8, pos: ImVec2, cond: Cond = .None) #no_context {
        SetWindowPos(name, *pos, cond);
    }
}

#if OS == .WINDOWS {
    SetWindowPos :: (pos: ImVec2, cond: Cond = .None) #no_context {
        SetWindowPos(*pos, cond);
    }
}

#if OS == .WINDOWS {
    SetWindowPos :: (name: *u8, pos: ImVec2, cond: Cond = .None) #no_context {
        SetWindowPos(name, *pos, cond);
    }
}

#if OS != .WINDOWS {
    SetWindowSize :: (size: ImVec2, cond: Cond = .None) #no_context {
        SetWindowSize(*size, cond);
    }
}

#if OS != .WINDOWS {
    SetWindowSize :: (name: *u8, size: ImVec2, cond: Cond = .None) #no_context {
        SetWindowSize(name, *size, cond);
    }
}

#if OS == .WINDOWS {
    SetWindowSize :: (size: ImVec2, cond: Cond = .None) #no_context {
        SetWindowSize(*size, cond);
    }
}

#if OS == .WINDOWS {
    SetWindowSize :: (name: *u8, size: ImVec2, cond: Cond = .None) #no_context {
        SetWindowSize(name, *size, cond);
    }
}

#if OS != .WINDOWS {
    Text :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = Text_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    Text :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = Text_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TextColored :: (col: ImVec4, fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) *col }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (col: *ImVec4, fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = TextColored_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TextColored :: (col: ImVec4, fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) *col }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (col: *ImVec4, fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = TextColored_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TextDisabled :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = TextDisabled_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TextDisabled :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = TextDisabled_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TextUnformatted :: (text: string) #no_context {
        TextUnformatted(text.data, text.data + text.count);
    }
}

#if OS == .WINDOWS {
    TextUnformatted :: (text: string) #no_context {
        TextUnformatted(text.data, text.data + text.count);
    }
}

#if OS != .WINDOWS {
    TextWrapped :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = TextWrapped_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TextWrapped :: (fmt: string, __args: ..Any) {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [1] Varargs.Argument = .[.{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (fmt: *u8, __args: ..Any) -> void #c_call;
        function: Native_Format = TextWrapped_CFormat;

        Varargs.invoke_void(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TreeNode :: (str_id: *u8, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) str_id }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (str_id: *u8, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNode_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TreeNode :: (ptr_id: *void, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) ptr_id }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNode_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TreeNode :: (str_id: *u8, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) str_id }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (str_id: *u8, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNode_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TreeNode :: (ptr_id: *void, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [2] Varargs.Argument = .[.{ kind = 6, value = cast(u64) ptr_id }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (ptr_id: *void, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNode_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TreeNodeEx :: (str_id: *u8, flags: TreeNodeFlags, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [3] Varargs.Argument = .[.{ kind = 6, value = cast(u64) str_id }, .{ kind = 1, value = cast(u64) flags }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (str_id: *u8, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNodeEx_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    TreeNodeEx :: (ptr_id: *void, flags: TreeNodeFlags, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [3] Varargs.Argument = .[.{ kind = 6, value = cast(u64) ptr_id }, .{ kind = 1, value = cast(u64) flags }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (ptr_id: *void, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNodeEx_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TreeNodeEx :: (str_id: *u8, flags: TreeNodeFlags, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [3] Varargs.Argument = .[.{ kind = 6, value = cast(u64) str_id }, .{ kind = 1, value = cast(u64) flags }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (str_id: *u8, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNodeEx_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS == .WINDOWS {
    TreeNodeEx :: (ptr_id: *void, flags: TreeNodeFlags, fmt: string, __args: ..Any) -> bool {
        cformat := Native.terminated_copy(fmt);

        defer Native.release(cformat);
        fixed: [3] Varargs.Argument = .[.{ kind = 6, value = cast(u64) ptr_id }, .{ kind = 1, value = cast(u64) flags }, .{ kind = 6, value = cast(u64) cformat }];
        Native_Format :: #type (ptr_id: *void, flags: TreeNodeFlags, fmt: *u8, __args: ..Any) -> bool #c_call;
        function: Native_Format = TreeNodeEx_CFormat;

        return Varargs.invoke_bool(cast(*void) function, fixed, __args);
    }
}

#if OS != .WINDOWS {
    VSliderFloat :: (label: *u8, size: ImVec2, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #no_context {
        return VSliderFloat(label, *size, v, v_min, v_max, format, flags);
    }
}

#if OS == .WINDOWS {
    VSliderFloat :: (label: *u8, size: ImVec2, v: *float, v_min: float, v_max: float, format: *u8 = "%.3f", flags: SliderFlags = .None) -> bool #no_context {
        return VSliderFloat(label, *size, v, v_min, v_max, format, flags);
    }
}

#if OS != .WINDOWS {
    VSliderInt :: (label: *u8, size: ImVec2, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #no_context {
        return VSliderInt(label, *size, v, v_min, v_max, format, flags);
    }
}

#if OS == .WINDOWS {
    VSliderInt :: (label: *u8, size: ImVec2, v: *s32, v_min: s32, v_max: s32, format: *u8 = "%d", flags: SliderFlags = .None) -> bool #no_context {
        return VSliderInt(label, *size, v, v_min, v_max, format, flags);
    }
}

#if OS != .WINDOWS {
    VSliderScalar :: (label: *u8, size: ImVec2, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #no_context {
        return VSliderScalar(label, *size, data_type, p_data, p_min, p_max, format, flags);
    }
}

#if OS == .WINDOWS {
    VSliderScalar :: (label: *u8, size: ImVec2, data_type: DataType, p_data: *void, p_min: *void, p_max: *void, format: *u8 = null, flags: SliderFlags = .None) -> bool #no_context {
        return VSliderScalar(label, *size, data_type, p_data, p_min, p_max, format, flags);
    }
}
