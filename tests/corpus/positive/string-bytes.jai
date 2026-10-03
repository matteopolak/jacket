main :: () -> int { text := "B\0C"; return text.count + cast(int) text[2] - 50; }
