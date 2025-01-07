import 'package:nostr_sdk/nostr_sdk.dart';

void event() {
  final keys = Keys.generate();

  // Compose custom event
  final customEvent =
      EventBuilder(kind: 1111, content: "").signWithKeys(keys: keys);

  // Compose text note
  final textnoteEvent =
      EventBuilder.textNote(content: "Hello").signWithKeys(keys: keys);

  // Compose reply to above text note
  final replyEvent = EventBuilder.textNote(content: "Reply to hello")
      .tag(tag: Tag.parse(tag: ['e', textnoteEvent.id().toHex()]))
      .signWithKeys(keys: keys);

  // Compose POW event
  final powEvent = EventBuilder.textNote(content: "Another reply with POW")
      .tag(tag: Tag.parse(tag: ['e', textnoteEvent.id().toHex()]))
      .pow(difficulty: 20)
      .signWithKeys(keys: keys);
}
