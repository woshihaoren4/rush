---
--- Generated by EmmyLua(https://github.com/EmmyLua)
--- Created by bytedance.
--- DateTime: 2023/9/26 4:37 PM
---



local md =  require("lua_script/module")

function handle(req)
    md.show_req(req)
    return {message='success'}
end

return {handle_function="handle"}