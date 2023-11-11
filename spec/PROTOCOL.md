# Protocol

## Packet Structure

<div align="center">
  <picture>
    <source media="(prefers-color-scheme: light)" srcset="./assets/packet-diagram-light.svg">
    <source media="(prefers-color-scheme: dark)" srcset="./assets/packet-diagram-dark.svg">
    <img alt="packet diagram">
  </picture>
</div>

## Fields

### Version

The version field indicates what version of the protocol the packet is using. Any packets with unsupported protocol version should immediately be discarded. If a packet has a mismatched version number, then one of the following situations have occurred:

1. The peer does not speak the same kind of protocol
2. The peer does not support the protocol version
3. There exists data corruption in the packet

In any case, further communication should not be attempted.

### Packet Length

This field encodes the length of the entire packet in bytes from the start of the [Version field](#version) to the end of the [Payload](#payload). The value of this field is calculated by summing the length of  the header and the length of the payload. All packets must set this field equal to 8 or more. With 8 being the minimum size of any packet, assuming a zero-size payload.

### Header Checksum

> TODO: Examine the possibility of introducing Error Correction Codes here instead.

This field is responsible for ensuring that any error in the header is detected. The checksum must be the final modification made to the header before it is sent out and must be calculated while this field is set to 0. The algorithm used for calculating the checksum is the [Internet Checksum](https://en.wikipedia.org/wiki/Internet_checksum) algorithm.

### Flags

This field is a bit-flag field, ie. it holds flags important for the transmission of the packet. The current flags are:

|     Bit     |          Name           |
|:-----------:|:-----------------------:|
| `0... ....` | [More data](#more-data) |
| `.000 0000` |        Reserved         |

#### More Data

If the sender wishes to transmit a payload larger than what the [length field](#packet-length) permits, they can set this flag to indicate to the receiver that they should continue waiting for packets containing the rest of the payload. Upon receiving a packet with this flag set, the receiver should store the payload data from that packet in a buffer and keep listening for incoming packets. Any further packets with this flag set should have their payload data appended to the buffer. The packet containing the final part of the payload must have this flag unset so as to indicate that the payload has been fully transmitted.

### Packet ID

This field is a short-lived identification number for the packet. It is used to ensure the correct ordering of packets and in no way a unique identifier of the packet throughout the entire conversation. Such IDs need not be cryptographically secure. The next ID must be able to be predicted by any party, given that they know the previous ID.

### Payload

This is the final and usually the largest part of the packet and it contains the serialized data.
