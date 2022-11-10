#version 400

const int maxUvChannelCount = 10;
const int maxBoneCount = 50;

in vec3 position;
in vec3 normal;
in vec3 tangent;
in vec2 uvChannels[maxUvChannelCount];
in uvec4 boneIds;
in vec4 boneWeights;

uniform vec3 eyePosition;
uniform mat4 objectMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform mat4 normalMatrix;
uniform mat4 bones[maxBoneCount];

uniform int useAlbedoTexture;
uniform sampler2D albedoTexture;
uniform uint albedoTextureUvChannelId;

uniform int useNormalTexture;
uniform sampler2D normalTexture;
uniform uint normalTextureUvChannelId;

uniform int useDisplacementTexture;
uniform sampler2D displacementTexture;
uniform uint displacementTextureUvChannelId;

uniform float opacity;
uniform vec3 albedoColor;
uniform vec3 emissiveColor;
uniform vec3 shininessColor;

out vec4 vWorldPos;
out vec3 vNormal;
out vec2 vUvChannels[maxUvChannelCount];
out mat3 vTangentMatrix;
out mat3 vInvTangentMatrix;

void main()
{
	mat4 boneTransform = 
		bones[boneIds[0]] * boneWeights[0] +
		bones[boneIds[1]] * boneWeights[1] +
		bones[boneIds[2]] * boneWeights[2] +
		bones[boneIds[3]] * boneWeights[3];

	vNormal = normalize(mat3(normalMatrix) * mat3(boneTransform) * normal);

	vTangentMatrix[0] = vec3(normalize(mat3(normalMatrix) * mat3(boneTransform) * tangent));
	vTangentMatrix[2] = vNormal;
	vTangentMatrix[1] = normalize(cross(vTangentMatrix[2], vTangentMatrix[0]));
	vInvTangentMatrix = transpose(vTangentMatrix);

	for (int i = 0; i < maxUvChannelCount; ++i)
	{
		vUvChannels[i] = uvChannels[i];
	}

	vWorldPos = objectMatrix * boneTransform * vec4(position, 1.0f);
	gl_Position = projectionMatrix * viewMatrix * vWorldPos;
}
