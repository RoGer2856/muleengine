#version 400

const int maxUvChannelCount = 10;
const int maxBoneCount = 50;

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

in vec4 vWorldPos;
in vec3 vNormal;
in vec2 vUvChannels[maxUvChannelCount];
in mat3 vTangentMatrix;
in mat3 vInvTangentMatrix;

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

vec3 getNormal(vec2 texCoordsOffset) {
	if (useNormalTexture == 1) {
		vec3 normal = texture(
			normalTexture,
			vUvChannels[normalTextureUvChannelId] + texCoordsOffset
		).rgb * 2.0f - 1.0f;

		return normalize(vTangentMatrix * normal);
	} else {
		// the following line is equivalent with the consequitive line
		return vNormal;
		// return normalize(vTangentMatrix * vec3(0.0f, 0.0f, 1.0f));
	}
}

// tsViewDir: texture space view dir
vec2 parallaxMapping(
	in vec3 tsViewDir,
	in vec2 texCoords,
	in float layers,
	in float parallaxScale,
	out float parallaxHeight) {

	// determine optimal number of layers
	const float minLayers = 10.0f;
	const float maxLayers = 15.0f;
	float numLayers = mix(maxLayers, minLayers, abs(dot(vec3(0.0f, 0.0f, 1.0f), tsViewDir)));
	numLayers = layers;

	// height of each layer
	float layerHeight = 1.0f / numLayers;
	// current depth of the layer
	float curLayerHeight = 0.0f;
	// shift of texture coordinates for each layer
	vec2 dtex = parallaxScale * tsViewDir.xy / tsViewDir.z / numLayers;

	// current texture coordinates
	vec2 currentTextureCoords = texCoords;

	// depth from heightmap
	float heightFromTexture = texture(displacementTexture, currentTextureCoords).r;

	// while point is above the surface
	while(heightFromTexture > curLayerHeight)
	{
		// to the next layer
		curLayerHeight += layerHeight;
		// shift of texture coordinates
		currentTextureCoords += dtex;
		// new depth from heightmap
		heightFromTexture = texture(displacementTexture, currentTextureCoords).r;
	}

	///////////////////////////////////////////////////////////

	// previous texture coordinates
	vec2 prevTCoords = currentTextureCoords - dtex;

	// heights for linear interpolation
	float nextH = heightFromTexture - curLayerHeight;
	float prevH = texture(displacementTexture, prevTCoords).r - curLayerHeight + layerHeight;

	// proportions for linear interpolation
	float weight = nextH / (nextH - prevH);

	// interpolation of texture coordinates
	vec2 finalTexCoords = prevTCoords * weight + currentTextureCoords * (1.0f - weight);

	// interpolation of depth values
	parallaxHeight = curLayerHeight + prevH * weight + nextH * (1.0f - weight);

	// return result
	return finalTexCoords - texCoords;
}

void main() {
	vec3 viewDir = normalize(vWorldPos.xyz - eyePosition);

	vec2 texCoordsOffset = vec2(0.0f);
	if (useDisplacementTexture == 1.0f)
	{
		float parallaxHeight;
		vec3 tsViewDir = vInvTangentMatrix * viewDir;
		texCoordsOffset = parallaxMapping(
			tsViewDir,
			vUvChannels[displacementTextureUvChannelId],
			50,
			0.05,
			parallaxHeight
		);
	}

	vec3 albedo = vec3(1.0f, 1.0f, 1.0f);
	vec3 ambient = vec3(0.1f, 0.1f, 0.1f);

	vec4 tmp = getAlbedoColor(texCoordsOffset);
	albedo = vec3(tmp);
	float alpha = tmp.a;
	if (alpha < 0.05) {
		discard;
	}

	vec3 normal = getNormal(texCoordsOffset);

	vec3 lightIntensity = vec3(0.1f, 0.1f, 0.1f);

	vec3 lightDir0 = vec3(1.2f, -0.8f, -1.0f);
	vec3 lightColor0 = vec3(1.0f, 1.0f, 1.0f);
	lightIntensity += lightColor0 * clamp(dot(normal, -normalize(lightDir0)), 0.0f, 1.0f);

	vec3 lightDir1 = vec3(-1.0f, 1.0f, 1.0f);
	vec3 lightColor1 = vec3(0.1f, 0.1f, 0.4f);
	lightIntensity += lightColor1 * clamp(dot(normal, -normalize(lightDir1)), 0.0f, 1.0f);

	vec3 resultColor = (albedo * lightIntensity + ambient) * albedoColor;
	fragColor = max(vec4(resultColor, alpha), vec4(emissiveColor, alpha));
}
