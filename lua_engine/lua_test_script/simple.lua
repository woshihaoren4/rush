
local md = require("../lua_test_script/module")

function handle(req)
    for k, v in pairs(req) do
        md.show(k,v)
    end
    local resp = {code=0,message="success"}
    return resp
end


function handle(req)
    local resp = {}

    if req.source == "online" then
        resp.message = "线上渠道"
    elseif req.source == "offline" then
        resp.message = "线下渠道"
    else
        resp.message = "未知渠道:"+req.source
    end

    return {handle_function="handle"}
end