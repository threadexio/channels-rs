-- https://www.wireshark.org/docs/wsdg_html_chunked/wsluarm_modules.html

proto = Proto("channels-rs", "channels-rs Protocol")

proto.fields.version  = ProtoField.uint16("channels.version",  "Version")
proto.fields.length   = ProtoField.uint16("channels.len",      "Length")
proto.fields.checksum = ProtoField.uint16("channels.checksum", "Checksum", base.HEX)
proto.fields.payload  = ProtoField.bytes("channels.payload", "Payload",  base.SPACE)

-- Layers
proto.fields.id       = ProtoField.uint16("channels.id", "ID")

function proto.dissector(buf, pinfo, tree)
	pinfo.cols.protocol = "channels-rs"
	local subtree = tree:add(proto, buf(), "channels-rs Protocol")

	local version  = buf(0, 2)
	local length   = buf(2, 2)
	local checksum = buf(4, 2)

	-- ID Layer
	local id       = buf(6, 2)
	pinfo.cols.info:prepend("ID=" .. id:uint() .. " ")

	local header_length = 8
	local payload  = buf(header_length)

	if length:uint() ~= buf:len() then
		subtree:add("[Bad packet] Invalid length!")
	end

	subtree:add_packet_field(proto.fields.version, version, ENC_BIG_ENDIAN)

	local length_tree = subtree:add_packet_field(proto.fields.length, length, ENC_BIG_ENDIAN)
	length_tree:add("[Header length]: "  .. header_length)
	length_tree:add("[Payload length]: " .. (buf:len() - header_length))

	subtree:add_packet_field(proto.fields.checksum, checksum, ENC_BIG_ENDIAN)

	-- Layers
	subtree:add_packet_field(proto.fields.id,       id,       ENC_BIG_ENDIAN)

	subtree:add_packet_field(proto.fields.payload,  payload,  ENC_BIG_ENDIAN)
end

tcp_table = DissectorTable.get("tcp.port")
tcp_table:add(10000,proto)
