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

### version

The version field indicates what version of the protocol the packet is using. Any packets with unsupported protocol version should immediately be discarded. If a packet has a mismatched version number, then one of the following situations have occurred:

1. The peer does not speak the same kind of protocol
2. The peer does not support the protocol version
3. There exists data corruption in the packet

In any case, further communication should not be attempted.

This document is the specification of the version `0x42` of the protocol. That is the value this field must have.

### flags

This field is a 2 bit unsigned number that holds flags for the frame. Currently no flags are used and as such this field must be 0.

### frame_num

This field is a wrapping 6 bit unsigned number that identifies the frame. It allows telling when a frame has not be received. For example, if using this protocol on top of UDP and assuming that each UDP packet holds exactly one frame, then it is possible to detect dropped packets and incorrect ordering of packets. The field starts with a value of 0 and for every frame is incremented by 1, wrapping on overflow.

### checksum

This field is responsible for ensuring that any error in the header is detected. The checksum must be the final modification made to the header before it is sent out and must be calculated while this field is set to 0. The checksum must be calculated from the start of the [version](#version) field to the end of the [data_len](#data_len) field. It does _not_ include the [payload](#payload). The algorithm used for calculating the checksum is the [Internet Checksum](https://en.wikipedia.org/wiki/Internet_checksum) algorithm.

### data_len

This field specifies the length of the [payload](#payload) that follows. It is a 32-bit little-endian unsigned number. The value of this field is the length of the payload of the frame.

### payload

This is the final and usually the largest part of the packet and contains actual data.
