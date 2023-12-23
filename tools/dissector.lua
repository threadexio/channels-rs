-- https://www.wireshark.org/docs/wsdg_html_chunked/wsluarm_modules.html

proto = Proto("channels-rs", "channels-rs Protocol")

bytes_left_in_packet = 0

fields = {
	{
		["field"] = ProtoField.uint16("channels.version", "Version", base.HEX),
		["length"] = 2,
		["on_add"] = function(buf, data, pinfo, tree)
			if data:uint() ~= 0xfd3f then
				tree:add("Invalid version!")
			end
		end
	},
	{
		["field"] = ProtoField.uint16("channels.length", "Length"),
		["length"] = 2,
		["on_add"] = function(buf, data, pinfo, tree)
			local HEADER_SIZE = 8

			local packet_len = data:uint()
			local payload_len = packet_len - HEADER_SIZE

			if packet_len < HEADER_SIZE then
				tree:add("Invalid packet length! (< HEADER_SIZE)")
			else
				tree:add(string.format("[Header length]:  " .. HEADER_SIZE))
				tree:add(string.format("[Payload length]: " .. payload_len))
			end

			if buf:len() < packet_len then
				bytes_left_in_packet = packet_len - buf:len()
				tree:add(string.format("[Bytes left in packet]: ".. bytes_left_in_packet))

				pinfo.cols.info:prepend("┌ ")
			else
				pinfo.cols.info:prepend("• ")
				bytes_left_in_packet = 0
			end
		end
	},
	{
		["field"] = ProtoField.uint16("channels.checksum", "Checksum", base.HEX),
		["length"] = 2,
	},
	{
		["field"] = ProtoField.uint8("channels.flags", "Flags", base.HEX),
		["length"] = 1,
		["on_add"] = function(buf, data, pinfo, tree)
			local flags_raw = data:uint()

			local flag_names = {
				"Reserved",
				"Reserved",
				"Reserved",
				"Reserved",
				"Reserved",
				"Reserved",
				"Reserved",
				"More Data",
			}

			for bit_num, name in ipairs(flag_names) do
				bit_num = bit_num - 1 -- start at 0

				local bit_mask = bit.lshift(1, bit_num)

				local is_set = 0
				if bit.band(flags_raw, bit_mask) ~= 0 then
					is_set = 1
				end

				local l_padding = 7 - bit_num
				local r_padding = bit_num

				local str = string.rep(".", l_padding) .. is_set .. string.rep(".", r_padding)
				str = string.sub(str, 1, 4) .. " " .. string.sub(str, 5, 8) -- add a space in the middle

				str = str .. string.format(" = 0x%02x", bit_mask * is_set) -- show the hex value of the flag

				local is_set_str
				if is_set ~= 0 then
					is_set_str = "Set"
				else
					is_set_str = "Not Set"
				end

				tree:add(string.format("%s: %-10s: %s", str, name, is_set_str))
			end
		end,
	},
	{
		["field"] = ProtoField.uint8("channels.id", "ID"),
		["length"] = 1,
		["on_add"] = function(buf, data, pinfo, tree)
			pinfo.cols.info:prepend(string.format("ID=%-3d | ", data:uint()))
		end
	},
	{
		["field"] = ProtoField.bytes("channels.payload", "Payload", base.SPACE),
	},
}

for i, p in ipairs(fields) do
	proto.fields[i] = p["field"]
end

function proto.dissector(buf, pinfo, tree)
	pinfo.cols.protocol = "channels-rs"
	local subtree = tree:add(proto, buf(), "channels-rs Protocol")

	if bytes_left_in_packet > 0 then
		bytes_left_in_packet = bytes_left_in_packet - buf:len()

		subtree:add(string.format("[Bytes left in packet]: ".. bytes_left_in_packet))

		if bytes_left_in_packet == 0 then
			pinfo.cols.info:prepend("Ignore | └ ")
		else
			pinfo.cols.info:prepend("Ignore | │ ")
		end

		return;
	else
		bytes_left_in_packet = 0
	end

	local pos = 0
	for i, p in ipairs(fields) do
		local start
		if p["start"] ~= nil then
			start = p["start"](buf)
		else
			start = pos
		end

		local data
		if p["length"] ~= nil then
			data = buf(start, p["length"])
			pos = pos + p["length"]
		else
			data = buf(start)
		end

		local field = p["field"]
		local field_tree = subtree:add_packet_field(proto.fields[i], data, ENC_BIG_ENDIAN)

		if p["on_add"] ~= nil then
			p["on_add"](buf, data, pinfo, field_tree)
		end
	end
end

tcp_table = DissectorTable.get("tcp.port")

ports = {
	10000,
	10001,
	10002,
	10003,
	13942
}

for _, port in pairs(ports) do
	tcp_table:add(port, proto)
end
