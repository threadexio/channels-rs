-- https://www.wireshark.org/docs/wsdg_html_chunked/wsluarm_modules.html

proto = Proto("channels-rs", "channels-rs Protocol")

proto.fields.version  = ProtoField.uint16("channels.version",  "Version",  base.HEX)
proto.fields.length   = ProtoField.uint16("channels.len",      "Length")
proto.fields.checksum = ProtoField.uint16("channels.checksum", "Checksum", base.HEX)
proto.fields.flags    = ProtoField.uint8("channels.flags",     "Flags",    base.HEX)
proto.fields.id       = ProtoField.uint8("channels.id",        "ID")
proto.fields.payload  = ProtoField.bytes ("channels.payload",  "Payload",  base.SPACE)

packet_is_in_group = false
bytes_left_in_payload = nil

function proto.dissector(buf, pinfo, tree)
	pinfo.cols.protocol = "channels-rs"
	local subtree = tree:add(proto, buf(), "channels-rs Protocol")

	local version  = buf(0, 2)
	local length   = buf(2, 2)
	local checksum = buf(4, 2)
	local flags    = buf(6, 1)
	local id       = buf(7, 1)

	local header_length = 8
	local payload = buf(header_length)

	pinfo.cols.info:prepend("ID=" .. id:uint() .. " ")

	if bytes_left_in_payload ~= nil then
		if bytes_left_in_payload <= 0 then
			bytes_left_in_payload = nil
		else
			bytes_left_in_payload = bytes_left_in_payload - buf:len()

			pinfo.cols.info:prepend("**Ignore** ")
		end
	else
		if buf:len() < length:uint() then
			-- this means that the channels packet was split up
			bytes_left_in_payload = length:uint() - buf:len()
		end
	end

	local more_data_flag_set = bitand(flags:uint(), bitshl(1, 7)) ~= 0

	subtree:add_packet_field(proto.fields.version, version, ENC_BIG_ENDIAN)

	local length_tree = subtree:add_packet_field(proto.fields.length, length, ENC_BIG_ENDIAN)
	length_tree:add("[Header length]: "  .. header_length)
	length_tree:add("[Payload length]: " .. (buf:len() - header_length))

	subtree:add_packet_field(proto.fields.checksum, checksum, ENC_BIG_ENDIAN)

	local flags_tree = subtree:add_packet_field(proto.fields.flags, flags, ENC_BIG_ENDIAN)
	add_flag_to_subtree(flags_tree, "More Data", 7, more_data_flag_set)

	subtree:add_packet_field(proto.fields.id,       id,       ENC_BIG_ENDIAN)
	subtree:add_packet_field(proto.fields.payload,  payload,  ENC_BIG_ENDIAN)

	local packet_group_prefix = ""
	if more_data_flag_set then
		if packet_is_in_group then
			packet_group_prefix = "│ "
		else
			packet_group_prefix = "┌ "
			packet_is_in_group = true
		end
	else
		if packet_is_in_group then
			packet_group_prefix = "└ "
			packet_is_in_group = false
		else
			packet_group_prefix = "· "
		end
	end
	pinfo.cols.info:prepend(packet_group_prefix)
end

tcp_table = DissectorTable.get("tcp.port")
tcp_table:add(10000, proto)
tcp_table:add(10001, proto)

function add_flag_to_subtree(tree, name, bit, is_set)
	local result = ""

	local lpad = 7 - bit
	for i = 1,lpad do
		result = "." .. result
	end

	if is_set then
		result = result .. "1"
	else
		result = result .. "0"
	end

	local rpad = 7 - lpad
	for i = 1,rpad do
		result = result .. "."
	end

	result = string.sub(result, 0, 4) .. " " .. string.sub(result, 5, 8)

	result = result .. " = " .. name .. ": "

	if is_set then
		result = result .. "Set"
	else
		result = result .. "Not Set"
	end

	tree:add(result)
end

function bitand(a, b)
	local result = 0
	local bitval = 1
	while a > 0 and b > 0 do
	  if a % 2 == 1 and b % 2 == 1 then -- test the rightmost bits
		  result = result + bitval      -- set the current bit
	  end
	  bitval = bitval * 2 -- shift left
	  a = math.floor(a/2) -- shift right
	  b = math.floor(b/2)
	end
	return result
end

function bitshl(a, b)
	return a * (2^b)
end

function bitshr(a, b)
	return math.floor(a / (2^b))
end
