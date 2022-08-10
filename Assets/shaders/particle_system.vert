out vec2 vTexCoords;

void main()
{
	vTexCoords = texCoords;
	gl_Position = projectionMatrix
		* modelviewMatrix
		* mat4(transform0, transform1, transform2, transform3)
		* vec4(position, 1.0);
}
