#version 130

uniform mat4 objectMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform mat3 normalMatrix;
uniform mat4 bones[50];

in vec3 position;
in vec3 tangent;
in vec2 texCoords;
in vec3 normal;
in ivec4 boneIds;
in vec4 boneWeights;

out vec4 vWorldPos;
out vec3 vNormal;
out vec2 vTexCoords;
out mat3 vTangentMatrix;
out mat3 vInvTangentMatrix;

void main()
{
	mat4 boneTransform =
		bones[int(boneIds[0])] * boneWeights[0] +
		bones[int(boneIds[1])] * boneWeights[1] +
		bones[int(boneIds[2])] * boneWeights[2] +
		bones[int(boneIds[3])] * boneWeights[3];

	vNormal = normalize(normalMatrix * mat3(boneTransform) * normal);

	vTangentMatrix[0] = normalize(normalMatrix * mat3(boneTransform) * tangent);
	vTangentMatrix[2] = normalize(vNormal);
	vTangentMatrix[1] = normalize(cross(vTangentMatrix[2], vTangentMatrix[0]));
	vInvTangentMatrix = transpose(vTangentMatrix);

	vTexCoords = texCoords;
	vWorldPos = objectMatrix * boneTransform * vec4(position, 1.0f);
	gl_Position = projectionMatrix * viewMatrix * vWorldPos;
}
