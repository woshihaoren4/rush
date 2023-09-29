local md =  require("lua_script/module")

function handle(req)
    md.show_req(req)
    return {message='success'}
end

return {handle_function="handle"}