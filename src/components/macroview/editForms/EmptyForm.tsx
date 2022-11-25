import { Flex, Text } from '@chakra-ui/react'

const EmptyForm = () => {
  return (
    <Flex h="100vh" justifyContent="center" alignItems="center">
      <Text
        fontWeight="semibold"
        fontSize={['sm', 'md', 'lg']}
        w="75%"
        textAlign="center"
      >
        Select an Element to edit
      </Text>
    </Flex>
  )
}

export default EmptyForm
