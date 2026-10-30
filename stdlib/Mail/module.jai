// Mail: sending email through an SMTP server, plus a small text obfuscation helper.
//
// Fill in a `Mail_Server` and a `Mail_Message` and call `mail_send`, which delivers the message with libcurl and returns
// whether the server accepted it. It blocks until the exchange is over, reports failures through `context.logger` and
// its result rather than stopping the program, and borrows every string it is given. See `mail_send` for what is
// checked up front and how TLS is used. `scramble` is a reversible XOR mapping for keeping a string out of plain view;
// it is not encryption.
//
// ```jai
// text := scramble("hunter2", "salt");
// assert(scramble(text, "salt") == "hunter2");
// ```

Mail_Server :: Impl.Mail_Server;
Mail_Message :: Impl.Mail_Message;
mail_send :: Impl.mail_send;
scramble :: Transform.scramble;

#scope_file
Impl :: #import "internal/net/mail";
Transform :: #import "internal/net/mail-scramble";
