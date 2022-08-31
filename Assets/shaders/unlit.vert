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
uniform mat3 normalMatrix;
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

out vec3 vNormal;
out vec2 vUvChannels[maxUvChannelCount];

void main()
{
	mat4 boneTransform = 
		bones[boneIds[0]] * boneWeights[0] +
		bones[boneIds[1]] * boneWeights[1] +
		bones[boneIds[2]] * boneWeights[2] +
		bones[boneIds[3]] * boneWeights[3];

	for (int i = 0; i < maxUvChannelCount; ++i)
	{
		vUvChannels[i] = uvChannels[i];
	}
	vNormal = normalMatrix * mat3(boneTransform) * normal;
	gl_Position = projectionMatrix * viewMatrix * objectMatrix * boneTransform * vec4(position, 1.0f);
}
