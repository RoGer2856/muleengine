
-- Walls
position = me2Vector3:new(0, 0, 0)
scale = me2Vector3:new(10, 10, 10)

load3DModel('demo/wall/wallTextured.fbx', 'main', 'pp_color', position, scale)
position = position + me2Vector3:new(25, 0, 0)
load3DModel('demo/wall/wallNormal.fbx', 'main', 'pp_color', position, scale)
position = position + me2Vector3:new(25, 0, 0)
load3DModel('demo/wall/wallFull.fbx', 'main', 'pp_color', position, scale)

load3DModel('demo/wall/wallTextured.fbx', 'main', 'pp_color', position + me2Vector3:new(-20, 0, 30), scale)
load3DModel('demo/wall/wallFull.fbx', 'main', 'pp_color', position + me2Vector3:new(0, 0, 30), scale)

position = position + me2Vector3:new(30, 0, 0)
load3DModel('demo/wall/wallSimple.fbx', 'main', 'pp_color', position, scale)
position = position + me2Vector3:new(25, 0, 0)
load3DModel('demo/wall/wallNormalWoAlbedo.fbx', 'main', 'pp_color', position, scale)
position = position + me2Vector3:new(25, 0, 0)
load3DModel('demo/wall/wallFullWoAlbedo.fbx', 'main', 'pp_color', position, scale)

load3DModel('demo/wall/wallSimple.fbx', 'main', 'pp_color', position + me2Vector3:new(-20, 0, 30), scale)
load3DModel('demo/wall/wallFullWoAlbedo.fbx', 'main', 'pp_color', position + me2Vector3:new(0, 0, 30), scale)
-- end of Walls

position = me2Vector3:new(0, 0, 0)
scale = me2Vector3:new(1, 1, 1)

--[[
load3DModel('sponza/sponza.fbx', 'main', 'pp_color', position, scale)
--load3DModel('sponza/sponza.fbx', 'main', 'pp_depth', position + me2Vector3:new(50, -10, 0), scale)
--]]
