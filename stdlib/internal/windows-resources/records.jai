ICONDIR :: struct {
    Reserved1: W.WORD;
    ResourceType: W.WORD;
    ImageCount: W.WORD;
}

ICONDIRENTRY :: struct {
    Width: u8;
    Height: u8;
    Colors: u8;
    Reserved2: u8;
    Planes: W.WORD;
    BitsPerPixel: W.WORD;
    dwBytesInResource: W.DWORD;
    dwImageOffset: W.DWORD;
}
