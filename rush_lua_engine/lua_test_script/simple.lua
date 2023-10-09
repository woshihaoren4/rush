
local md = require("../lua_test_script/module")

function handle(req)
    for k, v in pairs(req) do
        md.show(k,v)
    end
    local resp = {code=0,message="success"}
    return resp
end