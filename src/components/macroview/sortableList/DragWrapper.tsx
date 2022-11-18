import { DragHandleIcon } from '@chakra-ui/icons'
import { Box, HStack, useColorModeValue } from '@chakra-ui/react'
import React from 'react'

type Props = {
    id: number
    isSmall: boolean
    children: any
}

const DragWrapper = ({id, isSmall, children}: Props) => {
const bg = useColorModeValue('white', 'gray.800')
  const dividerColour = useColorModeValue('gray.400', 'gray.600')

  return (
    <HStack
      w={isSmall ? 'fit-content' : '100%'}
      border="1px"
      borderColor={dividerColour}
      rounded="md"
      spacing="0px"
      bg={bg}
      sx={{ cursor: 'auto' }}
    >
      <Box
        borderRight="1px"
        borderColor={dividerColour}
        p="4px"
        h="full"
        sx={{ cursor: 'grabbing' }}
      >
        <DragHandleIcon w={4} h={8} />
      </Box>
        {children}
    </HStack>
  )
}

export default DragWrapper