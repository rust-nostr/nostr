import 'package:nostr_sdk/nostr_sdk.dart';

void event() {
  final keys = Keys.generate();

  // Compose custom event
  final customEvent =
      EventBuilder(kind: 1111, content: "").signWithKeys(keys: keys);
  print(customEvent.asJson());

  // Compose text note
  final textNoteEvent =
      EventBuilder.textNote(content: "Hello").signWithKeys(keys: keys);

  // Compose reply to above text note
  final replyEvent = EventBuilder.textNote(content: "Reply to hello")
      .tag(tag: Tag.parse(tag: ['e', textNoteEvent.id().toHex()]))
      .signWithKeys(keys: keys);
  print(replyEvent.asJson());

  // Compose POW event
  final powEvent = EventBuilder.textNote(content: "Another reply with POW")
      .tag(tag: Tag.parse(tag: ['e', textNoteEvent.id().toHex()]))
      .pow(difficulty: 20)
      .signWithKeys(keys: keys);
  print(powEvent.asJson());
}
