
fgColor = me2Vector3:new(0, 0, 0)
bgColor = me2Vector3:new(1, 1, 1)
scale = me2Vector2:new(1, 1)
anchorType = AnchorType.Center
pivot = me2Vector2:new(0.5, 0.5)
position = me2Vector2:new(0, 0)

setCurrentSlide(0)

addLabel("Text from LUA", fgColor, bgColor, scale, anchorType, pivot, position)
addLabel("öüóőúéáűÖÜÓŐÚÉÁŰ", fgColor, bgColor, scale, anchorType, pivot, position + me2Vector2:new(0, -50))

setCurrentSlide(1)

addLabel("2nd slide", fgColor, bgColor, scale, anchorType, pivot, position)

setCurrentSlide(2)

addImage("slides/Linux_kernel_and_OpenGL_video_games.svg.png", scale, anchorType, pivot, position)
