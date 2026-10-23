// The HTML and CSV layouts for a mix of speakers: a turn runs through a speaker's instants,
// instants without a speaker are skipped without ending it, and consecutive spoken lines or
// directions share a paragraph.
#import "Basic";
S :: #import "Subtitles";

INPUT :: "[1]\n: s1\n= ALICE\n: s2\n= ALICE\nLine1\n\nLine2\n: s3\n= BOB\n++ dir first\nspoken after\n: s4\n= A\none\n= _\nx\n= A\ntwo\n= B\nthree\n= _\n= A\nfour\n: s5\n= A\n++ d1\n= A\n++ d2\none\n= B\n++ d3\n: s6\n= X 1\nt1\n= Y 2\n= X 3\n";

EXPECTED_HTML :: #string END
<head><style>.title          {text-align:left; margin-inline: 15vw; font-weight:800;    font-size:24px; }</style>
<style>.speaker-bold   {text-align:left; margin-inline: 35vw; font-weight:750;    font-size:16px;}</style>
<style>.speaker-nonbold{text-align:left; margin-inline: 35vw; font-weight:400;    font-size:16px;}</style>
<style>.dialog-bold    {text-align:left; margin-inline: 25vw; font-weight:400;    font-size:16px;}</style>
<style>.dialog-nonbold {text-align:left; margin-inline: 25vw; font-weight:100;    font-size:16px;}</style>
<style>.direction      {text-align:left; margin-inline: 15vw; font-style:italic;}</style>
<style>.keep-together  {break-inside:avoid;}</style>
</head>
<body>
<br><p></p><br>
<div class="keep-together">

<p class="title">s1</p>

<p class="speaker-nonbold"><b>ALICE</b></p>
</div><br><p></p><br>
<div class="keep-together">

<p class="title">s2</p>

<p class="speaker-nonbold"><b>ALICE</b></p>
<p class="dialog-nonbold">Line1<br>Line2</p>
</div><br><p></p><br>
<div class="keep-together">

<p class="title">s3</p>

<p class="speaker-nonbold"><b>BOB</b></p>
<p class="direction">dir first</p><p class="dialog-nonbold">spoken after</p>
</div><br><p></p><br>
<div class="keep-together">

<p class="title">s4</p>

<p class="speaker-nonbold"><b>A</b></p>
<p class="dialog-nonbold">one<br>two</p>

<p class="speaker-bold"><b>B</b></p>
<p class="dialog-bold">three</p>

<p class="speaker-nonbold"><b>A</b></p>
<p class="dialog-nonbold">four</p>
</div><br><p></p><br>
<div class="keep-together">

<p class="title">s5</p>

<p class="speaker-nonbold"><b>A</b></p>
<p class="direction">d1<br>d2</p><p class="dialog-nonbold">one</p>

<p class="speaker-bold"><b>B</b></p>
<p class="direction">d3</p>
</div><br><p></p><br>
<div class="keep-together">

<p class="title">s6</p>

<p class="speaker-nonbold"><b>X</b></p>
<p class="dialog-nonbold">t1</p>

<p class="speaker-nonbold"><b>Y</b></p>

<p class="speaker-nonbold"><b>X</b></p>
</div>
</body>
END

EXPECTED_CSV :: #string END
Line #,Conversation,Character,Dialogue,Directions,Wav Filename,Conversation Filename
262,s2,"ALICE","Line1
Line2","",s2_01.wav,s2.wav
263,s3,"BOB","spoken after","dir first",s3_01.wav,s3.wav
264,s4,"A","one
two","",s4_01.wav,s4.wav
265,s4,"B","three","",s4_02.wav,s4.wav
266,s4,"A","four","",s4_03.wav,s4.wav
267,s5,"A","one","d1
d2",s5_01.wav,s5.wav
268,s6,"X","t1","",s6_01.wav,s6.wav
END

main :: () {
    ok, subs := S.load_subtitles_from_memory(copy_string(INPUT), "turns");

    assert(ok);

    all: [..] *S.Subtitles_Section;
    for *subs.all_sections array_add(*all, it);

    builder: String_Builder;

    S.format_sections_as_html(*builder, all, "B");
    html := builder_to_string(*builder);

    assert(html == EXPECTED_HTML);

    reset(*builder);
    S.format_sections_as_csv(*builder, all);
    csv := builder_to_string(*builder);

    assert(csv == EXPECTED_CSV);

    print("ok\n");
}
