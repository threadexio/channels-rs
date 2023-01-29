-- https://www.wireshark.org/docs/wsdg_html_chunked/wsluarm_modules.html

proto = Proto("channels-rs", "channels-rs Protocol")

proto.fields.version  = ProtoField.uint16("channels.version",  "Version")
proto.fields.id       = ProtoField.uint16("channels.id",       "ID")
proto.fields.length   = ProtoField.uint16("channels.len",      "Length")
proto.fields.checksum = ProtoField.uint16("channels.checksum", "Checksum", base.HEX)
proto.fields.payload  = ProtoField.bytes("channels.payload",   "Payload",  base.SPACE)

header_length = 8

function proto.dissector(buf, pinfo, tree)
	local subtree = tree:add(proto, buf(), "channels-rs Protocol")

	local version  = buf(0, 2)
	local id       = buf(2, 2)
	local length   = buf(4, 2)
	local checksum = buf(6, 2)
	local payload  = buf(header_length)

	pinfo.cols.protocol = "channels-rs"
	pinfo.cols.info:prepend("ID=" .. id:uint() .. " ")

	if length:uint() ~= buf:len() then
		subtree:add("[Bad packet] Invalid length!")
	end

	subtree:add_packet_field(proto.fields.version, version, ENC_BIG_ENDIAN)
	subtree:add_packet_field(proto.fields.id,      id,      ENC_BIG_ENDIAN)

	local length_tree = subtree:add_packet_field(proto.fields.length, length, ENC_BIG_ENDIAN)
	length_tree:add("[Header length]: "  .. header_length)
	length_tree:add("[Payload length]: " .. (buf:len() - header_length))

	subtree:add_packet_field(proto.fields.checksum, checksum, ENC_BIG_ENDIAN)
	subtree:add_packet_field(proto.fields.payload,  payload,  ENC_BIG_ENDIAN)
end

tcp_table = DissectorTable.get("tcp.port")
tcp_table:add(10000,proto)
