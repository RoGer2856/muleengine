#version 400

const int maxUvChannelCount = 10;
const int maxBoneCount = 50;

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

in vec3 vNormal;
in vec2 vUvChannels[maxUvChannelCount];

out vec4 fragColor;

vec4 getAlbedoColor(vec2 texCoordsOffset) {
	if (useAlbedoTexture == 1) {
		return texture(
			albedoTexture,
			vUvChannels[albedoTextureUvChannelId] + texCoordsOffset
		);
	} else {
		return vec4(1.0f);
	}
}

void main()
{
	vec3 albedo = vec3(1.0, 1.0, 1.0);

	vec2 texCoordsOffset = vec2(0.0f, 0.0f);

	vec4 tmp = getAlbedoColor(texCoordsOffset);
	albedo = vec3(tmp);
	float alpha = tmp.a;

	fragColor = vec4(max(albedo, emissiveColor) * albedoColor, alpha);
}
